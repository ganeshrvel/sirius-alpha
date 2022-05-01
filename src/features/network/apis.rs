use crate::common::api_client::sirius_proxima::{ApiResponse, PingResponse, SIRIUS_PROXIMA_CLIENT};
use crate::common::models::sirius_proxima_api::SiriusProximaPing;
use crate::constants::headers::{HeaderKeys, HeaderValues};
use crate::EnvValues;
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    pub static ref network_apis: Apis = Apis::new();
}

#[derive(Clone, Copy, Debug)]
pub struct Apis;

impl Apis {
    pub fn ping(self) -> ApiResponse<PingResponse> {
        let json_data = SiriusProximaPing::new()?;
        let mut headers = HashMap::new();
        headers.insert(
            HeaderKeys::DEVICE_ID,
            json_data.device.details.device_id.as_str(),
        );
        headers.insert(EnvValues::API_TOKEN_KEY, EnvValues::API_SECRET_TOKEN);
        headers.insert(HeaderKeys::CONTENT_TYPE, HeaderValues::APPLICATION_JSON);

        SIRIUS_PROXIMA_CLIENT.put::<PingResponse, _, _, _>(
            "/api/v1/sirius_alpha/ping",
            &json_data,
            Some(headers),
            None,
        )
    }

    pub const fn new() -> Self {
        Self
    }
}
