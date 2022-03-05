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

mod tm1637;

#[macro_use]
extern crate dotenv_codegen;

// use std::net::ToSocketAddrs;
use std::fmt::format;
use std::net::{TcpStream, ToSocketAddrs};
use std::ptr::null_mut;
use std::sync::Mutex;
use std::{sync::Arc, thread, thread::sleep, time::Duration};

use anyhow::{bail, Result};
// use embedded_svc::http::client::{Client, Request};
// use embedded_svc::http::client::{Client, Request, Response};
// use embedded_svc::http::status::OK;
// use embedded_svc::http::{SendHeaders, Status as httpStatus};
use embedded_svc::ping::Ping;
use embedded_svc::wifi::{
    ClientConfiguration, ClientConnectionStatus, ClientIpStatus, ClientStatus, Configuration,
    Status,
};
use embedded_svc::{ipv4, ipv4::Ipv4Addr, wifi::Wifi};
// use esp_idf_hal::{delay::};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::{delay, gpio};

//use embedded_hal::blocking::delay::DelayUs;

// use esp_idf_hal::peripherals::Peripherals;

// use embedded_hal::blocking::delay::DelayMs;
// use embedded_hal::digital::v2::OutputPin;

// use esp_idf_hal::prelude::*;
// use esp_idf_hal::{delay, gpio};
//
// use esp_idf_hal::adc;
// use esp_idf_hal::i2c;
// use esp_idf_hal::prelude::*;
// use esp_idf_hal::spi;

// use esp_idf_sys::esp;
// use esp_idf_sys::{self, c_types};

use embedded_hal::digital::v2::OutputPin;

use embedded_hal::blocking::delay::DelayUs;
//
// use esp_idf_hal::delay::{Ets, FreeRtos};
//
//
// use esp_idf_hal::delay::TickType;

// use embedded_hal::prelude::_embedded_hal_blocking_delay_DelayUs;
// use esp_idf_hal::delay::Ets;

//
// use embedded_hal::adc::OneShot;
// use embedded_hal::blocking::delay::DelayMs;
//
// use embedded_svc::eth;
// use embedded_svc::eth::{Eth, TransitionalState};
// use embedded_svc::httpd::registry::*;
// use embedded_svc::httpd::*;
// use embedded_svc::io;
// use embedded_svc::mqtt::client::{Publish, QoS};
// use embedded_svc::sys_time::SystemTime;
// use embedded_svc::timer::TimerService;
// use embedded_svc::timer::*;
// use embedded_svc::wifi::*;

// use esp_idf_svc::http::client::{EspHttpClient, EspHttpClientConfiguration, EspHttpRequest};
// use embedded_svc::event_bus::EventBus;
// use embedded_svc::utils::nonblocking::Asyncify;
// use esp_idf_svc::eventloop::{EspSubscription, System};
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
// use esp_idf_sys::*;
use crate::tm1637::TM1637;
use log::info;
use serde::{Deserialize, Serialize};
// use tm1637::TM1637;

// use std::net::TcpStream;
// use http::Request;

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

    thread::Builder::new()
        .stack_size(32768)
        .spawn(move || {
            unsafe {
                esp_idf_sys::vTaskPrioritySet(null_mut(), 1_u32 as c_uint);
            }

            let peripherals = Peripherals::take().unwrap();
            let pins: gpio::Pins = peripherals.pins;

            // let (clk_pin, dio_pin) = (27, 23);
            //
            // let bit_delay_fn = Box::from(|| sleep(Duration::from_micros(10)));
            // let tm1637display = setup_gpio(clk_pin, dio_pin, bit_delay_fn);
            //
            // // display "1 2 3 4"
            // let data: [u8; 4] = [
            //     TM1637Adapter::encode_digit(1),
            //     TM1637Adapter::encode_digit(2),
            //     TM1637Adapter::encode_digit(3),
            //     TM1637Adapter::encode_digit(4),
            // ];
            // tm1637display.write_segments_raw(&data, 0);


            let mut led_g32 = pins.gpio32.into_output().unwrap();
            let mut led_g25 = pins.gpio25.into_output().unwrap();
            let mut led_g26 = pins.gpio26.into_output().unwrap();
            //let mut led_g12 = pins.gpio12.into_output().unwrap();
            // let mut buzzer_g25 = pins.gpio25.into_output().unwrap();
            let mut led_g27 = pins.gpio27.into_output().unwrap();
            let mut led_g13 = pins.gpio13.into_input_output().unwrap();


            // pins.gpio0.split(&mut rcc.apb2);

           // let dd = embedded_hal::blocking::delay::DelayUs::delay_us();
// Ets
      //      Ets
           // let dd =   Ets::delay_us(1000);

            //let mut delay = Delay::new(cp.SYST, clocks);
            // let mut t = NoDelay{};
            // let d = t.delay_us(1000);

          //  let mut  d =  FreeRtos.delay_us(1000 as u8);//
            //embedded_hal::blocking::delay::DelayUs{}

            //let mut d = delay::ets_delay_us()//Ets{}.delay_us(1000_u8); //::new(cp.SYST, clocks);

            //TM1637

            // let mut t = TickType::from(Duration::from_micros(1000));

            // FreeRtos::from(Ets::);

            let mut tm = TM1637::new(&mut led_g27,&mut led_g13);

            tm.init().unwrap(); // append `.unwrap()` to catch and handle exceptions in cost of extra ROM size
            tm.clear().unwrap();


           // tm.print_hex(3, &[0]);

           //tm.print_raw(3, &[11]);


            let mut counter = 0;

            loop {
                println!("counter: {}", counter.to_string());
                println!("counter >> 5: {}", counter >> 5);

                tm.print_hex(0, &[counter, counter + 1]);
                tm.print_hex(1, &[counter, counter + 2]);
                tm.print_hex(2, &[counter, counter + 3]);
                tm.print_raw(3, &[counter]);
                tm.set_brightness(counter >> 5);


                counter += 1;

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
                            println!("Text of page is {}", response.text().unwrap());


                            led_g32.set_high().unwrap();
                            thread::sleep(Duration::from_secs(2));
                            led_g32.set_low().unwrap();

                            led_g25.set_high().unwrap();
                            thread::sleep(Duration::from_secs(2));
                            led_g25.set_low().unwrap();

                            led_g26.set_high().unwrap();
                            thread::sleep(Duration::from_secs(2));
                            led_g26.set_low().unwrap();


                            // buzzer_g25.set_high().unwrap();
                            // thread::sleep(Duration::from_secs(3));
                            // buzzer_g25.set_low().unwrap();

                        }
                        Err(err) => {
                            println!("--- Couldn't fetch page, will sleep: {}", err.to_string());
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

                    thread::sleep(Duration::from_secs(2));
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
