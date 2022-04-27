use crate::common::api_client::sirius_proxima::{ApiResponse, SIRIUS_PROXIMA_CLIENT};
use crate::common::errors::api_errors::{ApiClientError, ApiResponseError};
use crate::common::errors::wifi_errors::WifiError;
use crate::common::models::sirius_proxima_api::{Device, Health};
use crate::constants::default_values::DefaultValues;
use crate::constants::segment_display_text::SegmentDisplayText;
use crate::features::peripheral_feature::PeripheralTx;
use crate::GpioPinValue::{High, Low};
use crate::{CommonError, PeripheralFeature, PeripheralKind, WifiAdaptor};
use embedded_svc::wifi::{ClientConnectionStatus, ClientIpStatus, ClientStatus, Status, Wifi};
use esp_idf_sys::c_types::c_uint;
use log::error;
use serde::de::DeserializeOwned;
use std::ptr::null_mut;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
pub struct NetworkFeature {
    is_network_connected: bool,
    is_wifi_ip_resolved: bool,
}

pub const STACK_SIZE: usize = 32768_u32 as usize;

impl NetworkFeature {
    fn network_response_to_segment_display_error<T>(self, response: &ApiResponse<T>) -> Option<&str>
    where
        T: DeserializeOwned,
    {
        return match &response {
            Ok(d) => None,
            Err(e) => {
                let matched_api_res_err: Option<&str> = match e.downcast_ref::<ApiResponseError>() {
                    None => None,
                    Some(ApiResponseError::InternalServerError(_, _, _)) => {
                        Some(SegmentDisplayText::ERR_503)
                    }
                    Some(
                        ApiResponseError::SiteNotFound(_, _) | ApiResponseError::NotFound(_, _, _),
                    ) => Some(SegmentDisplayText::ERR_404),
                    Some(ApiResponseError::BadRequest(_, _, _)) => {
                        Some(SegmentDisplayText::ERR_400)
                    }
                };

                if matched_api_res_err.is_some() {
                    return matched_api_res_err;
                }

                let matched_api_client_err: Option<&str> = match e.downcast_ref::<ApiClientError>()
                {
                    None => None,
                    Some(ApiClientError::Response(_, _)) => Some(SegmentDisplayText::ERR_API),
                    Some(ApiClientError::JsonParsing(_, _)) => Some(SegmentDisplayText::ERR_JSON),
                };

                if matched_api_client_err.is_some() {
                    return matched_api_client_err;
                }

                None
            }
        };
    }

    fn connect(self, wifi_adaptor: &mut WifiAdaptor) -> anyhow::Result<()> {
        wifi_adaptor.connect()
    }

    const fn check_wifi_ip_resolved(self, status: &Status) -> bool {
        if let Status(
            ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(_))),
            _,
        ) = *status
        {
            return true;
        }

        false
    }

    fn run_net_connection_worker(&mut self, wifi_adaptor: &mut WifiAdaptor) {
        ////////////////////////////////////////////////////////////////////////////////////////////////
        ////////
        // <---if the network connectivity hasnt been established yet then connect to the network--->
        if !self.is_network_connected {
            let c = self.connect(wifi_adaptor);

            if let Err(e) = c {
                // connectivity issue, try again
                log::warn!(
                    "[E0008] [network feature] an error occured: '{}'.\n\
                                    Will try to connect again...",
                    e
                );

                thread::sleep(Duration::from_millis(
                    DefaultValues::WIFI_RECONNECTION_DELAY,
                ));

                return;
            }

            // marking [is_network_connected] as true since a wifi connection has been established
            self.is_network_connected = true;
        }

        // </---if the network connectivity hasnt been established yet then connect to the network--->
        ////////

        ////////////////////////////////////////////////////////////////////////////////////////////////
        ////////
        // <--- check if the wifi status has turned into the connected state and the ip address has been resolved. --->

        let esp_wifi = &wifi_adaptor.esp_wifi;
        let status = esp_wifi.get_status();
        if !self.check_wifi_ip_resolved(&status) {
            // mark [is_wifi_ip_resolved] as false if the ip hasnt been resolved yet
            self.is_wifi_ip_resolved = false;

            thread::sleep(Duration::from_millis(500));

            return;
        }

        // marking [is_wifi_ip_resolved] as true since an ip has been resolved
        self.is_wifi_ip_resolved = true;
        // </--- check if the wifi status has turned into the connected state and the ip address has been resolved. --->
        ////////
    }

    fn run_ping_api_worker(
        &mut self,
        display_tx: &Sender<Option<String>>,
        peripheral_tx: &PeripheralTx,
    ) -> anyhow::Result<()> {
        if !self.is_network_connected {
            log::debug!(
                "[network feature] [run_apis] waiting for the wifi \
                     connection to get established..."
            );

            let display_tx_res = display_tx.send(Some(SegmentDisplayText::ERR_NO_WIFI.to_owned()));
            if let Err(err) = display_tx_res {
                error!(
                    "[E0027a][run_ping_api_worker display_tx_res] {}",
                    err.to_string()
                );
            }

            // set [WifiConnectedLed] as low since the wifi is not connected
            PeripheralFeature::set_peripheral(peripheral_tx, PeripheralKind::WifiConnectedLed(Low));

            thread::sleep(Duration::from_millis(100));

            return Err(WifiError::NotConnected("E0017".to_owned()).into());
        }

        if !self.is_wifi_ip_resolved {
            log::debug!(
                "[network feature] [run_apis] waiting for the \
            ip to get resolved..."
            );

            let display_tx_res = display_tx.send(Some(SegmentDisplayText::ERR_NO_WIFI.to_owned()));
            if let Err(err) = display_tx_res {
                error!(
                    "[E0027b][run_ping_api_worker display_tx] {}",
                    err.to_string()
                );
            }

            // set [WifiConnectedLed] as low since the wifi is not connected
            PeripheralFeature::set_peripheral(peripheral_tx, PeripheralKind::WifiConnectedLed(Low));

            thread::sleep(Duration::from_millis(100));

            return Err(WifiError::UnresolvedIp("E0018".to_owned()).into());
        }

        // turn on [WifiConnectedLed] since the network connection has been established now
        PeripheralFeature::set_peripheral(peripheral_tx, PeripheralKind::WifiConnectedLed(High));

        // blink the [ProximaApiRequestLed] to indicate a network api request
        for i in 0_i32..5_i32 {
            if i % 2 == 0_i32 {
                PeripheralFeature::set_peripheral(
                    peripheral_tx,
                    PeripheralKind::ProximaApiRequestLed(High),
                );
            } else {
                PeripheralFeature::set_peripheral(
                    peripheral_tx,
                    PeripheralKind::ProximaApiRequestLed(Low),
                );
            }

            thread::sleep(Duration::from_millis(25));
        }
        // turn off [ProximaApiRequestLed]
        PeripheralFeature::set_peripheral(peripheral_tx, PeripheralKind::ProximaApiRequestLed(Low));

        // network request starts here
        // let resp = SIRIUS_PROXIMA_CLIENT.get::<Device>("/api/v1/sirius_alpha/ping");
        let resp = SIRIUS_PROXIMA_CLIENT.get::<Health>("/api/health");
        let segment_display_text = self.network_response_to_segment_display_error(&resp);

        if let Some(text) = segment_display_text {
            let res = display_tx.send(Some(text.to_owned()));
            if let Err(err) = res {
                error!("[E0027c][run_ping_api_worker] {}", err.to_string());
            }
        }

        Ok(())
    }

    fn start_netmanager_thread(
        this: Arc<Mutex<Self>>,
        wifi_adaptor: Arc<Mutex<WifiAdaptor>>,
        netmanager_condvar: Arc<Condvar>,
    ) -> std::io::Result<JoinHandle<anyhow::Result<()>>> {
        thread::Builder::new()
            .stack_size(STACK_SIZE)
            .spawn(move || -> anyhow::Result<()> {
                // SAFETY: ESP IDF related sys call
                unsafe {
                    esp_idf_sys::vTaskPrioritySet(null_mut(), 1_u32 as c_uint);
                }

                let timeout_duration = Duration::from_millis(1000);
                let mut last_exec_time = Instant::now();

                let mut this = this
                    .lock()
                    .map_err(|e| CommonError::MutexGuard("E0014".to_owned(), e.to_string()))?;

                let mut wifi_adaptor = wifi_adaptor
                    .lock()
                    .map_err(|e| CommonError::MutexGuard("E0012".to_owned(), e.to_string()))?;

                loop {
                    log::debug!("[start_netmanager_thread] entering into the next iteration...");

                    if Instant::now() - last_exec_time
                        >= Duration::from_millis(DefaultValues::NET_CONNECTION_MANAGER_THREAD_DELAY)
                    {
                        this.run_net_connection_worker(&mut wifi_adaptor);

                        last_exec_time = Instant::now();
                    } else {
                        log::debug!(
                            "[start_netmanager_thread] the threshold delay not completed yet, \
                            skipping this iteration..."
                        );
                    }

                    // Use condvar to release mutex and wait until signaled to start again
                    let (new_guard, _) = netmanager_condvar
                        .wait_timeout(this, timeout_duration)
                        .map_err(|e| CommonError::MutexGuard("E0015".to_owned(), e.to_string()))?;

                    this = new_guard;
                }
            })
    }

    pub fn start_workers_thread(
        this: Arc<Mutex<Self>>,
        worker_condvar: Arc<Condvar>,
        display_tx: Sender<Option<String>>,
        peripheral_tx: PeripheralTx,
    ) -> std::io::Result<JoinHandle<anyhow::Result<()>>> {
        thread::Builder::new()
            .stack_size(STACK_SIZE)
            .spawn(move || -> anyhow::Result<()> {
                let mut this = this
                    .lock()
                    .map_err(|e| CommonError::MutexGuard("E0013".to_owned(), e.to_string()))?;

                let timeout_duration = Duration::from_millis(1000);

                let mut last_exec_time = Instant::now();

                loop {
                    log::debug!("[start_workers_thread] entering into the next iteration...");

                    if Instant::now() - last_exec_time
                        >= Duration::from_millis(DefaultValues::APIS_THREAD_DELAY)
                    {
                        let res = this.run_ping_api_worker(&display_tx, &peripheral_tx);

                        // todo test this logic
                        // todo make sure that back to back api calls arent happening for success messages
                        // skip setting the [last_exec_time] if there were any errors in the API call
                        // unless the ping was successful we keep ignoring the threshold delay to make sure that our HTTP request goes through at the earliest possible
                        if res.is_ok() {
                            last_exec_time = Instant::now();
                        }
                    } else {
                        log::debug!(
                            "[start_netmanager_thread] the threshold delay not\
                             completed yet, skipping this iteration..."
                        );
                    }

                    // Sleep until signaled that the connection has been fixed
                    let (new_guard, _) = worker_condvar
                        .wait_timeout(this, timeout_duration)
                        .map_err(|e| CommonError::MutexGuard("E0016".to_owned(), e.to_string()))?;

                    this = new_guard;
                }
            })
    }

    pub fn start(
        this: &Arc<Mutex<Self>>,
        wifi_adaptor: &Arc<Mutex<WifiAdaptor>>,
        seg_display_tx: Sender<Option<String>>,
        peripheral_tx: PeripheralTx,
    ) -> anyhow::Result<()> {
        let self_cloned1 = Arc::clone(this);
        let self_cloned2 = Arc::clone(this);
        let wifi_adaptor_cloned1: Arc<Mutex<WifiAdaptor>> = Arc::clone(wifi_adaptor);

        let netmanager_thread_condvar = Arc::new(Condvar::new());
        let workers_thread_condvar = Arc::new(Condvar::new());

        Self::start_netmanager_thread(
            self_cloned1,
            wifi_adaptor_cloned1,
            netmanager_thread_condvar,
        )?;

        Self::start_workers_thread(
            self_cloned2,
            workers_thread_condvar,
            seg_display_tx,
            peripheral_tx,
        )?;

        Ok(())
    }

    pub const fn new() -> Self {
        Self {
            is_network_connected: false,
            is_wifi_ip_resolved: false,
        }
    }
}
