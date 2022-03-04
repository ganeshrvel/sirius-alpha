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

#[macro_use]
extern crate dotenv_codegen;

// use std::net::ToSocketAddrs;
use std::fmt::format;
use std::net::{TcpStream, ToSocketAddrs};
use std::ptr::null_mut;
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
// use esp_idf_svc::http::client::{EspHttpClient, EspHttpClientConfiguration, EspHttpRequest};
// use embedded_svc::event_bus::EventBus;
// use embedded_svc::utils::nonblocking::Asyncify;
// use esp_idf_svc::eventloop::{EspSubscription, System};
use esp_idf_svc::{
    // http::server::{EspHttpRequest, EspHttpServer},
    log::EspLogger,
    netif::EspNetifStack,
    nvs::EspDefaultNvs,
    ping,
    sysloop::EspSysLoopStack,
    wifi::EspWifi,
};
use esp_idf_sys::c_types::c_uint;
use esp_idf_sys::link_patches;
// use esp_idf_sys::*;
use log::info;
use serde::{Deserialize, Serialize};

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

fn wifi_f(
    netif_stack: Arc<EspNetifStack>,
    sys_loop_stack: Arc<EspSysLoopStack>,
    default_nvs: Arc<EspDefaultNvs>,
) -> anyhow::Result<()> {
    let mut wifi = EspWifi::new(netif_stack, sys_loop_stack, default_nvs).expect("Need wifi");

    unsafe {
        esp_idf_sys::esp_wifi_set_ps(esp_idf_sys::wifi_ps_type_t_WIFI_PS_NONE);
    }

    println!("Wifi created, about to scan");

    let ap_infos = wifi.scan().expect("Need scan results");

    let ours = ap_infos.into_iter().find(|a| a.ssid == WIFI_SSID);

    let channel = if let Some(ours) = ours {
        println!(
            "Found configured access point {} on channel {}",
            WIFI_SSID, ours.channel
        );
        Some(ours.channel)
    } else {
        println!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            WIFI_SSID
        );
        None
    };

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: WIFI_SSID.into(),
        password: WIFI_PASS.into(),
        channel,
        ..Default::default()
    }))
    .expect("Couldn't set configuration");

    println!("Wifi configuration set, about to get status");

    let status = wifi.get_status();

    if let Status(
        ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(_))),
        _,
    ) = status
    {
        println!("Wifi connected");

        thread::Builder::new()
            .stack_size(32768)
            .spawn(move || {
                unsafe {
                    esp_idf_sys::vTaskPrioritySet(null_mut(), 1u32 as c_uint);
                }
                loop {
                    let r = attohttpc::get(format!("{}/api/health", API_BASE_URL));

                    // let t = t.danger_accept_invalid_certs(true);
                    // let t = t.danger_accept_invalid_hostnames(true);
                    /*.danger_accept_invalid_certs(true).danger_accept_invalid_hostnames(true)*/

                    match r.send() {
                        Ok(response) => {
                            println!("Text of page is {}", response.text().unwrap());
                            return;
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
                    thread::sleep(Duration::from_secs(5));
                }
            })
            .expect("Failed to spawn thread");

        loop {
            // Trap execution here
            thread::sleep(Duration::from_millis(1000));
        }
    } else {
        println!("Unexpected Wifi status: {:?}", status);
        panic!();
    }

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
