#![deny(clippy::all)]
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

#[macro_use]
extern crate dotenv_codegen;

use crate::common::adaptors::network::WifiAdaptor;
use crate::common::errors::common_errors::CommonError;
use crate::common::errors::setup_errors::SetupError;
use crate::common::libs::tm1637::mappings::{Brightness, DisplayState};
use crate::common::libs::tm1637::{Tm1637, Tm1637BannerAutoScrollConfig};
use crate::constants::env_values::EnvValues;

use embedded_svc::sys_time::SystemTime;
use esp_idf_hal::gpio;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_sys::link_patches;
use log::{error, warn};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::constants::strings::Strings;
use crate::features::network_feature::NetworkFeature;
use crate::helpers::logs::fern_log::setup_logging;

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
    let system_time = esp_idf_svc::systime::EspSystemTime {};

    let wifi_adaptor = WifiAdaptor::new()?;
    let wifi_adaptor_arc = Arc::new(Mutex::new(wifi_adaptor));

    let (seg_display_tx, seg_display_rx): (Sender<Option<String>>, Receiver<Option<String>>) =
        std::sync::mpsc::channel();

    let segement_display_message: Option<String> = None;
    let segement_display_message_arc = Arc::new(Mutex::new(segement_display_message));
    let segement_display_message_arc_clone = Arc::clone(&segement_display_message_arc);

    let net_features = NetworkFeature::new();
    NetworkFeature::start(
        &Arc::new(Mutex::new(net_features)),
        &wifi_adaptor_arc,
        seg_display_tx,
    )?;

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

    let peripherals = Peripherals::take();
    match peripherals {
        None => {
            return Err(SetupError::Peripherals("E0030b", "'peripherals' is empty").into());
        }
        Some(peripherals) => {
            let pins: gpio::Pins = peripherals.pins;
            let mut clk_g27 = pins.gpio27.into_input_output()?;
            let mut dio_g13 = pins.gpio13.into_input_output()?;

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
                                if min_timer_seconds_to_display_counter
                                    % min_timer_seconds_to_display_size
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
        }
    }

    loop {
        thread::sleep(Duration::from_millis(5000));
    }
}

/*fn wifi_f(
    netif_stack: Arc<EspNetifStack>,
    sys_loop_stack: Arc<EspSysLoopStack>,
    default_nvs: Arc<EspDefaultNvs>,
) -> anyhow::Result<()> {
    let wifi = EspWifi::new(netif_stack, sys_loop_stack, default_nvs).expect("Need wifi");
    let wifi_arc = Arc::new(Mutex::new(wifi));

    let wifi_arc_clone = wifi_arc.clone();

    let shared_state = Arc::new(Mutex::new(0));
    let shared_state_clone1 = shared_state.clone();
    let shared_state_clone2 = shared_state.clone();

    unsafe {
        esp_idf_sys::esp_wifi_set_ps(esp_idf_sys::wifi_ps_type_t_WIFI_PS_NONE);
    }

    println!("Wifi created, about to scan");

    let ap_infos = wifi_arc_clone
        .lock()
        .unwrap()
        .scan()
        .expect("Need scan results");

    let ours = ap_infos.into_iter().find(|a| a.ssid == WIFI_SSID);

    let channel = if let Some(ours) = ours {
        println!(
            "Found configured access point {} on channel {}",
            WIFI_SSID,
            ours.channel.to_string()
        );
        Some(ours.channel)
    } else {
        println!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            WIFI_SSID
        );
        None
    };

    wifi_arc_clone
        .lock()
        .unwrap()
        .set_configuration(&Configuration::Client(ClientConfiguration {
            ssid: WIFI_SSID.into(),
            password: WIFI_PASS.into(),
            channel,
            ..Default::default()
        }))
        .expect("Couldn't set configuration");

    println!("Wifi configuration set, about to get status");

    let peripherals = Peripherals::take().unwrap();
    let pins: gpio::Pins = peripherals.pins;
    // let pins_arc = Arc::new(Mutex::new(pins));
    //
    // let pins_arc = pins_arc.lock().unwrap();
    let mut clk_g27 = pins.gpio27.into_input_output().unwrap();
    let mut dio_g13 = pins.gpio13.into_input_output().unwrap();

    let mut led_g32 = pins.gpio32.into_output().unwrap();
    let mut led_g25 = pins.gpio25.into_output().unwrap();
    let mut led_g26 = pins.gpio26.into_output().unwrap();

    //  let now = Instant::now();
    // // if now - t > Duration::from_millis(1000) {
    // //     t = now;
    // //     println!("{:?} thread 0", t);
    // //     //println!("thread 0");
    // // }

    let time_now = esp_idf_svc::systime::EspSystemTime {};

    thread::Builder::new() /*.stack_size(32768)*/
        .spawn(move || {
            let mut tm = TM1637::new(&mut clk_g27, &mut dio_g13);
            tm.set_display_state(DisplayState::On);
            tm.set_brightness(Brightness::L7);
            tm.clear();
            let mut show_colon = false;

            loop {
                /// SHARED_MEMORY.with(|val| println!("--7seg thread shared_state {}", *val.borrow()));
                // shared_state_clone1.lock().unwrap();
                let mut ss = shared_state_clone1.lock();
                match ss {
                    Err(ee) => {
                        println!("--7seg thread error {:?}", ee.to_string())
                    }
                    Ok(mut ssss) => {
                        println!("--7seg thread old shared_state {}", *ssss);

                        *ssss = *ssss + 1;

                        println!("--7seg thread new shared_state_clone1 {}", *ssss);
                    }
                }

                let mut pin_number = 0;

                let t = time_now.now();
                //println!("--- time_now {:?}", t);

                //todo remove this test block
                let t = t + Duration::from_secs(3590);
                //println!("--- new_time_now {:?}", t);

                //todo remove this test block

                let seconds = t.as_secs() % 60;
                let minutes = (t.as_secs() / 60) % 60;
                let hours = (t.as_secs() / 60) / 60;
                // println!("--- duration {:02}:{:02}", minutes, seconds);

                if hours < 1 {
                    let min_sec_t = format!("{:02}{:02}", minutes, seconds);

                    show_colon = !show_colon;

                    tm.print_string(&min_sec_t, show_colon, None).unwrap();

                    thread::sleep(Duration::from_millis(1000))
                } else {
                    let hour_min_sec_t = format!(
                        "{}{} {:02}{} {:02}{}",
                        hours, "h", minutes, "n", seconds, "c"
                    );

                    let c = Tm1637BannerAutoScrollConfig {
                        scroll_min_char_count: tm.display_size + 1,
                        delay_ms: 750,
                        min_char_count_to_be_displayed: tm.display_size,
                    };
                    tm.print_string(&hour_min_sec_t, false, Some(&c)).unwrap();

                    thread::sleep(Duration::from_millis(2000))
                }
            }
        });

    thread::Builder::new()
        .stack_size(32768)
        .spawn(move || {
            unsafe {
                esp_idf_sys::vTaskPrioritySet(null_mut(), 1_u32 as c_uint);
            }

            //let mut led_g12 = pins.gpio12.into_output().unwrap();
            // let mut buzzer_g25 = pins.gpio25.into_output().unwrap();

            loop {
                let status = wifi_arc_clone.clone().lock().unwrap().get_status();

                if let Status(
                    ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(_))),
                    _,
                ) = status
                {
                    println!("Wifi connected");

                    let r = attohttpc::get(format!("{}/api/health", API_BASE_URL));


                    // let t = t.danger_accept_invalid_certs(true);
                    // let t = t.danger_accept_invalid_hostnames(true);
                    /*.danger_accept_invalid_certs(true).danger_accept_invalid_hostnames(true)*/

                    match r.send() {
                        Ok(response) => {



                             let mut ss = shared_state.lock();

                            match ss {
                                Err (ee) => {
                                    println!("--wifi thread error {:?}", ee.to_string())
                                }
                                Ok  (mut ssss) => {
                                    println!("--wifi thread old shared_state {}", *ssss);

                                    *ssss = *ssss + 1;

                                    println!("--wifi thread new shared_state {}", *ssss);
                                }
                            }



                            // SHARED_MEMORY.with(|val|
                            //     println!("--wifi thread old shared_state {}", *val.borrow())
                            // );
                            //
                            // SHARED_MEMORY.with(|val| {
                            //     let v = *val.borrow();
                            //     *val.borrow_mut() = v+1;
                            // });
                            //
                            // SHARED_MEMORY.with(|val|
                            //     println!("--wifi thread new shared_state {}", *val.borrow())
                            // );

                             println!("-- response Text of page is {}", response.text().unwrap());


                            led_g32.set_high().unwrap();
                            thread::sleep(Duration::from_secs(1));
                            led_g32.set_low().unwrap();

                            led_g25.set_high().unwrap();
                            thread::sleep(Duration::from_secs(1));
                            led_g25.set_low().unwrap();

                            led_g26.set_high().unwrap();
                            thread::sleep(Duration::from_secs(1));
                            led_g26.set_low().unwrap();



                            // buzzer_g25.set_high().unwrap();
                            // thread::sleep(Duration::from_secs(3));
                            // buzzer_g25.set_low().unwrap();

                        }
                        Err(err) => {
                            println!("--- Couldn't fetch page, will sleep: {} {}", err.to_string(), API_BASE_URL);
                            println!("Maybe check your system time if you have certificate validation issues?");
                            // unsafe {
                            //     esp_idf_sys::settimeofday(
                            //         &timeval {
                            //             tv_sec: todo!(),
                            //             tv_usec: 0,
                            //         },
                            //         &timezone {
                            //             tz_minuteswest: 0,
                            //             tz_dsttime: 0,
                            //         },
                            //     );
                            // }
                        }
                    }

                    thread::sleep(Duration::from_secs(1));
                } else {
                    println!("Unexpected Wifi status: {:?}", status);

                    thread::sleep(Duration::from_secs(3));
                }
            }
        })?;

    Ok(())
    }
 */
// const fn check_status(status: &Status) -> bool {
//     use ClientConnectionStatus::Connected;
//     use ClientIpStatus::Done;
//     use ClientStatus::Started;
//     matches!(&status.0, Started(Connected(Done(_ip_settings))))
// }
//
// fn ping_f(ip_settings: &ipv4::ClientSettings) -> Result<()> {
//     info!("About to do some pings for {:?}", ip_settings);
//
//     let ping_summary = ping::EspPing::default().ping(
//         ip_settings.subnet.gateway,
//         &embedded_svc::ping::Configuration::default(),
//     )?;
//     if ping_summary.transmitted != ping_summary.received {
//         bail!(
//             "Pinging gateway {} resulted in timeouts",
//             ip_settings.subnet.gateway
//         );
//     }
//
//     info!("Pinging done");
//
//     Ok(())
// }
//
// fn print_startup_message() {
//     info!("");
//     info!("--------------------------------------------------------------");
//     info!("--------------------------------------------------------------");
//     info!("");
// }
