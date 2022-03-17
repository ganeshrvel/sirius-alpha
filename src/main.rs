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
    clippy::module_name_repetitions
)]

// mod tm1637;
// mod tmdd;
mod tm1637;

#[macro_use]
extern crate dotenv_codegen;

use std::ptr::null_mut;
use std::sync::Mutex;
use std::{sync::Arc, thread, time::Duration};

use embedded_svc::wifi::Wifi;
use embedded_svc::wifi::{
    ClientConfiguration, ClientConnectionStatus, ClientIpStatus, ClientStatus, Configuration,
    Status,
};
use esp_idf_hal::gpio;
use esp_idf_hal::peripherals::Peripherals;

use embedded_hal::digital::v2::OutputPin;

use embedded_svc::sys_time::SystemTime;

use std::time::Instant;

use crate::tm1637::{Brightness, DisplayState, SegmentBits, Tm1637BannerAutoScrollConfig, TM1637};
use esp_idf_svc::{
    // http::server::{EspHttpRequest, EspHttpServer},
    log::EspLogger,
    netif::EspNetifStack,
    nvs::EspDefaultNvs,
    sysloop::EspSysLoopStack,
    wifi::EspWifi,
};
use esp_idf_sys::c_types::c_uint;
use esp_idf_sys::link_patches;
use serde::{Deserialize, Serialize};

const WIFI_SSID: &str = dotenv!("WIFI_SSID");

const WIFI_PASS: &str = dotenv!("WIFI_PASS");

const API_TOKEN_KEY: &str = dotenv!("API_TOKEN_KEY");

const API_SECRET_TOKEN: &str = dotenv!("API_SECRET_TOKEN");

const API_BASE_URL: &str = dotenv!("API_BASE_URL");

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
pub struct SAlphaDeviceDetails {
    pub device_name: String,
    pub model: String,
    pub device_id: String,
    pub device_location: String,
    pub device_sdk: String,
    pub app_version: String,
}

// pub trait Serialize {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer;
// }
//
// impl Serialize for SAlphaDeviceDetails {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         serializer.serialize_i32(*self)
//     }
// }

fn main() -> anyhow::Result<()> {
    link_patches();
    EspLogger::initialize_default();

    let netif_stack = Arc::new(EspNetifStack::new()?);
    let sys_loop_stack = Arc::new(EspSysLoopStack::new()?);
    let default_nvs = Arc::new(EspDefaultNvs::new()?);

    let mut _wifi = wifi_f(netif_stack, sys_loop_stack, default_nvs)?;

    // Print the startup message, then spin for eternity so that the server does not
    // get dropped!
    //print_startup_message();

    // sleep(Duration::from_secs(7));
    //
    // let esp_client_config = EspHttpClientConfiguration{
    //     buffer_size: None,
    //     follow_redirects_policy: Default::default(),
    //     use_global_ca_store: false,
    //     crt_bundle_attach: None
    // };
    //

    // sleep(Duration::from_secs(10));
    //
    // let mut esp_client = EspHttpClient::new_default()?;
    // println!("{:?} --- status ", _wifi.get_status());
    // //
    // let mut esp_put = esp_client.put(String::from("https://www.google.com"))?;
    //
    // esp_put.set_header(API_TOKEN_KEY, API_SECRET_TOKEN);
    //
    // let r = SAlphaDeviceDetails {
    //     device_name: "".to_string(),
    //     model: "".to_string(),
    //     device_id: "".to_string(),
    //     device_location: "".to_string(),
    //     device_sdk: "".to_string(),
    //     app_version: "".to_string(),
    // };
    //
    // let j = serde_json::to_string(&r)?;
    //
    // let esp_req = esp_put.send_str(&j)?;
    //
    // println!("{:?} --- esp_req status_message", esp_req.status_message());
    //
    // // esp_client.request()
    // // esp_client.post()
    //
    // loop {
    //     sleep(Duration::from_secs(2));
    //     println!("{:?} --- status ", _wifi.get_status());
    // }

    Ok(())
}

struct NoDelay {}
impl embedded_hal::blocking::delay::DelayUs<u16> for NoDelay {
    fn delay_us(&mut self, us: u16) {}
}

fn wifi_f(
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

                let mut ss = shared_state_clone1.lock().unwrap();

                println!("--7seg thread old shared_state {}", *ss);

                *ss = *ss + 1;

                println!("--7seg thread new shared_state_clone1 {}", *ss);

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

                        let mut ss = shared_state.lock().unwrap();

                        println!("--wifi thread old shared_state {}", *ss);

                        *ss = *ss + 1;

                        println!("--wifi thread new shared_state {}", *ss);

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

                      //  println!("Text of page is {}", response.text().unwrap());


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
