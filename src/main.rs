#![deny(clippy::all, unused_must_use)]
#![warn(
    clippy::all,
    clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    clippy::print_stdout
)]
#![allow(
    clippy::missing_docs_in_private_items,
    clippy::future_not_send,
    clippy::implicit_return,
    clippy::similar_names,
    clippy::blanket_clippy_restriction_lints,
    clippy::module_name_repetitions,
    clippy::shadow_reuse,
    clippy::as_conversions,
    clippy::separated_literal_suffix,
    clippy::decimal_literal_representation,
    clippy::too_many_lines,
    clippy::integer_division,
    clippy::modulo_arithmetic,
    clippy::unused_self,
    clippy::integer_arithmetic
)]

mod common;
mod constants;
mod features;
mod helpers;
mod macros;
mod utils;
mod libs;

#[macro_use]
extern crate dotenv_codegen;

use crate::common::adaptors::network::WifiAdaptor;
use crate::common::errors::common_errors::CommonError;
use crate::common::errors::device_errors::DeviceError;
use crate::libs::tm1637::mappings::{Brightness, DisplayState, GpioPinValue};
use crate::libs::tm1637::{Tm1637, Tm1637BannerAutoScrollConfig};
use crate::constants::env_values::EnvValues;

use embedded_svc::sys_time::SystemTime;
use esp_idf_sys::link_patches;
use log::{error, warn};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::constants::strings::Strings;
use crate::features::network_feature::NetworkFeature;
use crate::features::peripheral_feature::{
    PeripheralFeature, PeripheralFeatureStartPins, PeripheralKind, PeripheralRx, PeripheralTx,
};
use crate::helpers::logs::fern_log::setup_logging;
use crate::GpioPinValue::High;

fn main() -> anyhow::Result<()> {
    link_patches();

    #[allow(clippy::print_stdout)]
    {
        println!("initializing the logger...");
    }
    setup_logging()?;

    log::debug!("-----------------");

    log::debug!("Launching {}...", Strings::APP_NAME);

    if let Err(e) = run() {
        log::error!("{:?}", e);

        return Err(e);
    }

    Ok(())
}

fn run() -> anyhow::Result<()> {
    let (peripheral_tx, peripheral_rx): (PeripheralTx, PeripheralRx) = std::sync::mpsc::channel();
    let per = PeripheralFeature::new()?;
    let peripheral_feature_start_pins = PeripheralFeatureStartPins {
        led_g23: per.out_g32,
        led_g25: per.out_g25,
        led_g26: per.out_g26,
    };
    PeripheralFeature::start(peripheral_feature_start_pins, peripheral_rx)?;
    PeripheralFeature::set_peripheral(&peripheral_tx, PeripheralKind::PowerOnLed(High));

    let system_time = esp_idf_svc::systime::EspSystemTime {};

    let wifi_adaptor = WifiAdaptor::new()?;
    let wifi_adaptor_arc = Arc::new(Mutex::new(wifi_adaptor));
    let (seg_display_tx, seg_display_rx): (Sender<Option<String>>, Receiver<Option<String>>) =
        std::sync::mpsc::channel();

    let net_features = NetworkFeature::new();
    NetworkFeature::start(
        &Arc::new(Mutex::new(net_features)),
        &wifi_adaptor_arc,
        seg_display_tx,
        peripheral_tx,
    )?;

    let segement_display_message: Option<String> = None;
    let segement_display_message_arc = Arc::new(Mutex::new(segement_display_message));
    let segement_display_message_arc_clone = Arc::clone(&segement_display_message_arc);
    thread::Builder::new().spawn(move || loop {
        match seg_display_rx.recv() {
            Ok(msg) => match segement_display_message_arc_clone.lock() {
                Ok(d) => {
                    let mut seg_text = d;

                    *seg_text = msg;
                }
                Err(e) => {
                    error!("[E0029a][seg_display_rx thread] {}", e.to_string());
                }
            },
            Err(e) => {
                error!("[E0029b][seg_display_rx thread] {}", e.to_string());
            }
        }
    })?;

    let mut clk_g27 = per.inout_g27;
    let mut dio_g13 = per.inout_g13;
    thread::Builder::new().spawn(move || {
        let mut tm = Tm1637::new(&mut clk_g27, &mut dio_g13);
        tm.set_display_state(DisplayState::On);
        tm.set_brightness(Brightness::L7);
        let tm_clear_res = tm.clear();
        if let Err(e) = tm_clear_res {
            error!("[E0031a][peripherals] {}", e.to_string());
        }

        // show the texts of the segment display after the defined number of loop iterations
        // this value is `5` when running time is less than `1 hour`
        // and `2` when running time is more than `1 hour`
        let mut min_timer_seconds_to_display_size = 5_i32;

        // if the [print_seg_text_on_display_count] is in multiples of[print_seg_text_on_display_cutoff_count] then print the [segement_display_message] to the display. This is to avoid error texts from taking hogging up the display
        let mut min_timer_seconds_to_display_counter = 0_i32;

        // for some strage reason adding a [thread::delay] is not freeing the [segement_display_message] hence not able to recieve messages via the [seg_display_rx] channel. This is a hack, a way around, it works and need to investigate why this is acting weird
        let mut next_delay = 0;

        let mut show_colon = false;

        loop {
            thread::sleep(Duration::from_millis(next_delay));

            let seg_text_res = segement_display_message_arc.lock();
            match seg_text_res {
                Ok(mut seg_text) => {
                    if let Some(msg) = &mut *seg_text {
                        if min_timer_seconds_to_display_counter % min_timer_seconds_to_display_size
                            == 0
                        {
                            let c = Tm1637BannerAutoScrollConfig {
                                scroll_min_char_count: tm.display_size + 1,
                                delay_ms: 750,
                                min_char_count_to_be_displayed: tm.display_size,
                            };

                            let tm_print_res = tm.print_string(msg, false, Some(&c), 0);
                            if let Err(e) = tm_print_res {
                                error!("[E0031b][peripherals] {}", e.to_string());
                            }

                            *seg_text = None;
                            min_timer_seconds_to_display_counter += 1_i32;
                            next_delay = 2000;

                            continue;
                        }
                    }
                }
                Err(e) => {
                    error!("[E0031c][peripherals] {}", e.to_string());
                }
            }

            min_timer_seconds_to_display_counter += 1_i32;
            let time_now = system_time.now();

            let seconds = time_now.as_secs() % 60;
            let minutes = (time_now.as_secs() / 60) % 60;
            let hours = (time_now.as_secs() / 60) / 60;

            // when running time is less than 1 hour then just show `00:00`
            if hours < 1 {
                let min_sec_t = format!("{:02}{:02}", minutes, seconds);

                show_colon = !show_colon;

                let res = tm.print_string(&min_sec_t, show_colon, None, 100_u16);
                if let Err(err) = res {
                    error!(
                        "[E0028a][segment display printing thread] {}",
                        err.to_string()
                    );
                }

                next_delay = 1000;
                min_timer_seconds_to_display_size = 5_i32;

                continue;
            }

            // when running time is more than 1 hour then show `00h 00n 00c`
            let hour_min_sec_t = format!(
                "{}{} {:02}{} {:02}{}",
                hours, "h", minutes, "n", seconds, "c"
            );
            let c = Tm1637BannerAutoScrollConfig {
                scroll_min_char_count: tm.display_size + 1,
                delay_ms: 750,
                min_char_count_to_be_displayed: tm.display_size,
            };

            let res = tm.print_string(&hour_min_sec_t, false, Some(&c), 0);
            if let Err(err) = res {
                error!(
                    "[E0028b][segment display printing thread] {}",
                    err.to_string()
                );
            }

            next_delay = 2000;
            min_timer_seconds_to_display_size = 2_i32;
        }
    })?;

    loop {
        thread::sleep(Duration::from_millis(5000));
    }
}
