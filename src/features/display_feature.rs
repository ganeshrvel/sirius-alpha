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
pub struct PeripheralsFeature {}

pub const STACK_SIZE: usize = 32768_u32 as usize;

impl PeripheralsFeature {


    pub fn start_tm1637_thread(){

    }


    pub fn start_tm1637_thread(){

    }

    pub const fn new() -> Self {
        Self {}
    }
}
