use crate::common::api_client::sirius_proxima::SIRIUS_PROXIMA_CLIENT;
use crate::common::errors::api_errors::{ApiClientError, ApiResponseError};
use crate::common::errors::wifi_errors::WifiError;
use crate::common::models::sirius_proxima_api::Health;
use crate::constants::default_values::DefaultValues;
use crate::features::network_feature;
use crate::{paniq, CommonError, EnvValues, WifiAdaptor};
use anyhow::private::kind::TraitKind;
use anyhow::{anyhow, Error};
use embedded_svc::wifi::{ClientConnectionStatus, ClientIpStatus, ClientStatus, Status, Wifi};
use esp_idf_sys::c_types::c_uint;
use std::ptr::null_mut;
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
    fn connect(&self, wifi_adaptor: &mut WifiAdaptor) -> anyhow::Result<()> {
        wifi_adaptor.connect()
    }

    const fn check_wifi_ip_resolved(&self, status: &Status) -> bool {
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

    fn run_ping_api_worker(&mut self) -> anyhow::Result<()> {
        if !self.is_network_connected {
            log::info!(
                "[network feature] [run_apis] waiting for the wifi \
                     connection to get established..."
            );

            thread::sleep(Duration::from_millis(100));

            return Err(WifiError::NotConnected("E0017".to_owned()).into());
        }

        if !self.is_wifi_ip_resolved {
            log::info!(
                "[network feature] [run_apis] waiting for the \
            ip to get resolved..."
            );

            thread::sleep(Duration::from_millis(100));

            return Err(WifiError::UnresolvedIp("E0018".to_owned()).into());
        }

        // let req = attohttpc::get(format!("{}/api/health", EnvValues::API_BASE_URL));
        // let req = req.danger_accept_invalid_certs(true);
        // let req = req.danger_accept_invalid_hostnames(true);
        //
        // match req.send() {
        //     Ok(response) => {
        //         println!("response text: {:?}", response.text())
        //     }
        //     Err(err) => {
        //         println!("response error: {:?}", err);
        //
        //         return Err(ApiClientError::Unknown("E0019".to_owned(), err.to_string()).into());
        //     }
        // }

        let resp = SIRIUS_PROXIMA_CLIENT.get::<Health>("/api/health");

        match &resp {
            Ok(d) => {
                println!("--- resp result {:?}", d);
                println!("--- resp result is_health_ok {:?}", d.is_health_ok);
            }
            Err(e) => {
                match e.downcast_ref::<ApiResponseError>() {
                    None => {}
                    Some(ApiResponseError::InternalServerError(_, _, _)) => {
                        unimplemented!(); //todo set the tm1673
                    }
                    Some(ApiResponseError::NotFound(_, _, _)) => {
                        unimplemented!(); //todo set the tm1673
                    }
                    Some(ApiResponseError::BadRequest(_, _, _)) => {
                        unimplemented!(); //todo set the tm1673
                    }
                };

                match e.downcast_ref::<ApiClientError>() {
                    None => {}
                    Some(ApiClientError::Response(_, _)) => {
                        unimplemented!(); //todo
                    }
                    Some(ApiClientError::JsonParsing(_, _)) => {
                        unimplemented!(); //todo
                    }
                }
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
                        let res = this.run_ping_api_worker();

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
    ) -> anyhow::Result<()> {
        let self_arc = this;
        let self_cloned1 = Arc::clone(self_arc);
        let self_cloned2 = Arc::clone(self_arc);
        let wifi_adaptor_cloned1: Arc<Mutex<WifiAdaptor>> = Arc::clone(wifi_adaptor);

        let netmanager_thread_condvar = Arc::new(Condvar::new());
        let workers_thread_condvar = Arc::new(Condvar::new());

        Self::start_netmanager_thread(
            self_cloned1,
            wifi_adaptor_cloned1,
            netmanager_thread_condvar,
        )?;

        Self::start_workers_thread(self_cloned2, workers_thread_condvar)?;

        Ok(())
    }

    pub const fn new() -> Self {
        Self {
            is_network_connected: false,
            is_wifi_ip_resolved: false,
        }
    }
}
