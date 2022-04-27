use crate::common::errors::wifi_errors::WifiError;
use crate::constants::default_values::DefaultValues;
use crate::constants::env_values::EnvValues;
use embedded_svc::wifi::{ClientConfiguration, Configuration, Wifi};
use esp_idf_svc::netif::{EspNetif, EspNetifStack, InterfaceConfiguration, InterfaceStack};
use esp_idf_svc::nvs::EspDefaultNvs;
use esp_idf_svc::sysloop::EspSysLoopStack;
use esp_idf_svc::wifi::EspWifi;
use esp_idf_sys::c_types::c_char;
use log::error;
use std::ffi::CString;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

pub struct WifiAdaptor {
    pub esp_wifi: EspWifi,
}

impl WifiAdaptor {
    pub fn connect(&mut self) -> anyhow::Result<()> {
        let wifi_channel: Option<u8>;
        loop {
            let channel = self.scan();

            if let Ok(c) = channel {
                wifi_channel = Some(c);
                break;
            }

            log::warn!("[network] the wifi scanning was unsuccessful. Will try again...");

            sleep(Duration::from_millis(
                DefaultValues::WIFI_RECONNECTION_DELAY,
            ));
        }

        self.esp_wifi
            .set_configuration(&Configuration::Client(ClientConfiguration {
                ssid: EnvValues::WIFI_SSID.into(),
                password: EnvValues::WIFI_PASS.into(),
                channel: wifi_channel,
                ..ClientConfiguration::default()
            }))
            .map_err(|e| WifiError::Configuration("E0004".to_owned(), e.to_string()))?;

        Ok(())
    }

    pub fn scan(&mut self) -> anyhow::Result<u8> {
        log::debug!("[network] starting wifi access point scanning...");

        let ap_infos = self
            .esp_wifi
            .scan()
            .map_err(|e| WifiError::Scanning("E0002".to_owned(), e.to_string()))?;

        let ap_info = ap_infos
            .into_iter()
            .find(|a| a.ssid == EnvValues::WIFI_SSID);

        let channel = if let Some(ap) = ap_info {
            log::debug!(
                "[network] found the configured access point {} on channel {}",
                EnvValues::WIFI_SSID,
                ap.channel.to_string()
            );
            ap.channel
        } else {
            log::error!(
                "[network] the configured access point `{}` was not found during the scanning",
                EnvValues::WIFI_SSID
            );
            return Err(
                WifiError::ApNotFound("E0006".to_owned(), EnvValues::WIFI_SSID.to_owned()).into(),
            );
        };

        Ok(channel)
    }

    pub fn new() -> anyhow::Result<Self> {
        let netif_stack = Arc::new(EspNetifStack::new()?);
        // let netif_stack_clone = Arc::<esp_idf_svc::netif::EspNetifStack>::clone(&netif_stack);
        //  let interface_stack_config = InterfaceStack::Sta.get_default_configuration();

        // let esp_netif = EspNetif::new(netif_stack_clone, &interface_stack_config)?;

        // esp_netif.set_hostname(EnvValues::DEVICE_ID)?;

        let sys_loop_stack = Arc::new(EspSysLoopStack::new()?);
        let default_nvs = Arc::new(EspDefaultNvs::new()?);

        let esp_wifi = EspWifi::new(netif_stack, sys_loop_stack, default_nvs)?;

        // netif_stack.

        // pub struct esp_netif_config {
        //     pub base: *const esp_netif_inherent_config_t,
        //     pub driver: *const esp_netif_driver_ifconfig_t,
        //     pub stack: *const esp_netif_netstack_config_t,
        // }

        // SAFETY: ESP IDF related sys call
        unsafe {
            esp_idf_sys::esp_wifi_set_ps(esp_idf_sys::wifi_ps_type_t_WIFI_PS_NONE);

            // let c = std::ffi::CString::new(EnvValues::DEVICE_ID.to_owned())?;
            // esp_idf_sys::tcpip_adapter_set_hostname(
            //     esp_idf_sys::tcpip_adapter_if_t_TCPIP_ADAPTER_IF_STA,
            //     c.as_ptr(),
            // );

            //let esp_netif_t = esp_idf_sys::esp_netif_new();
            //esp_idf_sys::esp_netif_set_hostname(esp_netif_t, c.as_ptr());
            //  esp_idf_sys::sethostname(
            //     c.as_ptr(),
            //     esp_idf_sys::tcpip_adapter_if_t_TCPIP_ADAPTER_IF_STA,
            // );
        }

        log::debug!("[network] wifi adaptor created");

        Ok(Self { esp_wifi })
    }
}
