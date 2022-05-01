use crate::common::errors::wifi_errors::WifiError;
use crate::constants::default_values::DefaultValues;
use crate::constants::env_values::EnvValues;
use embedded_svc::ipv4;
use embedded_svc::ipv4::DHCPClientSettings;
use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration, Wifi};
use esp_idf_svc::netif::EspNetifStack;
use esp_idf_svc::nvs::EspDefaultNvs;
use esp_idf_svc::sysloop::EspSysLoopStack;
use esp_idf_svc::wifi::EspWifi;
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
                DefaultValues::WIFI_RECONNECTION_DELAY_MS,
            ));
        }

        let dhcp_conf = DHCPClientSettings {
            hostname: Some(EnvValues::DEVICE_ID.to_owned()),
        };
        let ip_conf = ipv4::ClientConfiguration::DHCP(dhcp_conf);
        let cl = ClientConfiguration {
            ssid: EnvValues::WIFI_SSID.into(),
            password: EnvValues::WIFI_PASS.into(),
            channel: wifi_channel,
            bssid: None,
            auth_method: AuthMethod::default(),
            ip_conf: Some(ip_conf),
        };

        self.esp_wifi
            .set_configuration(&Configuration::Client(cl))
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
        let sys_loop_stack = Arc::new(EspSysLoopStack::new()?);
        let default_nvs = Arc::new(EspDefaultNvs::new()?);

        let esp_wifi = EspWifi::new(netif_stack, sys_loop_stack, default_nvs)?;

        // SAFETY: ESP IDF related sys call
        unsafe {
            esp_idf_sys::esp_wifi_set_ps(esp_idf_sys::wifi_ps_type_t_WIFI_PS_NONE);
        }

        log::debug!("[network] wifi adaptor created");

        Ok(Self { esp_wifi })
    }
}
