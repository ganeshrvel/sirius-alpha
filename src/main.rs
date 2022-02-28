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
use std::net::{TcpStream, ToSocketAddrs};
use std::{sync::Arc, thread::sleep, time::Duration};

use smol::{io, Async, Unblock};

use anyhow::{bail, Result};
// use embedded_svc::http::client::{Client, Request};
use embedded_svc::ping::Ping;
use embedded_svc::wifi::{
    ClientConfiguration, ClientConnectionStatus, ClientIpStatus, ClientStatus, Configuration,
    Status,
};
use embedded_svc::{ipv4, ipv4::Ipv4Addr, wifi::Wifi};
// use esp_idf_svc::http::client::EspHttpClient;
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
use esp_idf_sys::link_patches;
// use esp_idf_sys::*;
use log::info;
use smol::io::AsyncWriteExt;
// use std::net::TcpStream;
// use http::Request;

#[allow(dead_code)]
const WIFI_SSID: &str = dotenv!("WIFI_SSID");

#[allow(dead_code)]
const WIFI_PASS: &str = dotenv!("WIFI_PASS");

// const WIFI_CHAN: u8 = 6;
// const WIFI_CONN: u8 = 3;
// const DHCP_GTWY: Ipv4Addr = Ipv4Addr::new(10, 0, 0, 1);

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

    sleep(Duration::from_secs(7));

    smol::block_on(async {
        println!("==========");

        // Connect to http://example.com
        // let mut addrs = smol::unblock(move || ("example.com", 80).to_socket_addrs()).await?;
        // let addr = addrs.next().unwrap();
        // let mut stream = Async::<TcpStream>::connect(addr).await?;
        //
        // // Send an HTTP GET request.
        // let req = b"GET / HTTP/1.1\r\nHost: example.com\r\nConnection: close\r\n\r\n";
        // stream.write_all(req).await?;
        //
        // // Read the response and pipe it to the standard output.
        // let mut stdout = Unblock::new(std::io::stdout());
        // io::copy(&stream, &mut stdout).await?;
        Ok(())
    })

    // loop {
    //     sleep(Duration::from_secs(2));
    // }
}

fn wifi_f(
    netif_stack: Arc<EspNetifStack>,
    sys_loop_stack: Arc<EspSysLoopStack>,
    default_nvs: Arc<EspDefaultNvs>,
) -> Result<Box<EspWifi>> {
    let mut wifi = Box::new(EspWifi::new(netif_stack, sys_loop_stack, default_nvs)?);

    // info!("Wifi created, about to scan");
    //
    // let ap_infos = wifi.scan()?;
    //
    // let ours = ap_infos.into_iter().find(|a| a.ssid == WIFI_SSID);
    //
    // let channel = if let Some(ours) = ours {
    //     info!(
    //         "Found configured access point {} on channel {}",
    //         WIFI_SSID, ours.channel
    //     );
    //     Some(ours.channel)
    // } else {
    //     info!(
    //         "Configured access point {} not found during scanning, will go with unknown channel",
    //         WIFI_SSID
    //     );
    //     None
    // };

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: WIFI_SSID.into(),
        password: WIFI_PASS.into(),
        // channel,
        channel: None,
        ..embedded_svc::wifi::ClientConfiguration::default()
    }))?;

    info!("Wifi configuration set, about to get status");

    // let we = wifi.subscribe(|shared| {
    //
    //     // let status = wifi.get_status();
    //     println!("=====1 {:?}", s);
    // })?;

    // wifi.wait_status_with_timeout(Duration::from_secs(7), |status| !status.is_transitional())
    //     .map_err(|e| anyhow::anyhow!("Unexpected Wifi status: {:?}", e))?;
    //
    // let status = wifi.get_status();

    // wifi.wait_status(check_status);

    // println!("====={:?}", status.0);

    //  wifi.wait_status_with_timeout(Duration::from_secs(20),)

    // if let Status(
    //     ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(ip_settings))),
    //     ClientStatus::Started(ClientStatus::Done),
    // ) = status
    // {
    //     info!("Wifi connected");
    //
    //     ping_f(&ip_settings)?;
    // } else {
    //     bail!("Unexpected Wifi status: {:?}", status);
    // }

    Ok(wifi)
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
