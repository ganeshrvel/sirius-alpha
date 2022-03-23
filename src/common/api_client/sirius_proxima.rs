use crate::common::api_client::ApiClient;
use crate::common::errors::api_errors::{ApiClientError, ApiResponseError};
use crate::common::models::sirius_proxima_api::{
    SiriusProximaErrorResponse, SiriusProximaSuccessResponse,
};
use crate::constants::default_values::DefaultValues;
use crate::EnvValues;
use attohttpc::{Response, StatusCode};
use lazy_static::lazy_static;
use log::error;
use serde::de::DeserializeOwned;

lazy_static! {
    pub static ref SIRIUS_PROXIMA_CLIENT: SiriusProximaClient = SiriusProximaClient::new();
}

pub struct SiriusProximaClient {
    base_api_client: ApiClient,
}

enum ResponseType {
    OkResponse(String, StatusCode),
    ErrorResponse(String, StatusCode),
}

impl SiriusProximaClient {
    fn handle_response<T: DeserializeOwned>(
        &self,
        response: attohttpc::Result<Response>,
    ) -> anyhow::Result<T> {
        let resp_ok = match response {
            Ok(server_response) => {
                let is_success = server_response.is_success();
                let status_code = server_response.status();
                let resp_text = server_response.text();
                let resp_text_ok = match resp_text {
                    Ok(s) => s,
                    Err(e) => {
                        error!("[E0021][SiriusProximaClient] {}", e.to_string());

                        return Err(
                            ApiClientError::Response("E0021".to_owned(), e.to_string()).into()
                        );
                    }
                };

                if is_success {
                    ResponseType::OkResponse(resp_text_ok, status_code)
                } else {
                    ResponseType::ErrorResponse(resp_text_ok, status_code)
                }
            }
            Err(e) => {
                error!("[E0020][SiriusProximaClient] {}", e.to_string());

                return Err(ApiClientError::Response("E0020".to_owned(), e.to_string()).into());
            }
        };

        return match resp_ok {
            ResponseType::OkResponse(d, _) => {
                let resp_json = serde_json::from_str::<SiriusProximaSuccessResponse<T>>(&*d);
                let resp_json_ok = match resp_json {
                    Ok(s) => s,
                    Err(e) => {
                        error!("[E0022][SiriusProximaClient] {}", e.to_string());

                        return Err(
                            ApiClientError::JsonParsing("E0022".to_owned(), e.to_string()).into(),
                        );
                    }
                };

                Ok(resp_json_ok.data)
            }
            ResponseType::ErrorResponse(d, status_code) => {
                let resp_json = serde_json::from_str::<SiriusProximaErrorResponse>(&*d);
                let resp_json_ok = match resp_json {
                    Ok(s) => s,
                    Err(e) => {
                        error!("[E0023][SiriusProximaClient] {}", e.to_string());

                        return Err(
                            ApiClientError::JsonParsing("E0023".to_owned(), e.to_string()).into(),
                        );
                    }
                };

                let resp_json_err = resp_json_ok.error.unwrap_or_default();
                let resp_json_message = resp_json_ok.message.unwrap_or_default();

                error!("[E0024][SiriusProximaClient] Received an API response error. Error: {}, Message: {}", resp_json_err,resp_json_message );
                let mapped_res_error = match status_code {
                    StatusCode::BAD_REQUEST => ApiResponseError::BadRequest(
                        "E0024".to_owned(),
                        resp_json_err,
                        resp_json_message,
                    ),
                    StatusCode::NOT_FOUND => ApiResponseError::NotFound(
                        "E0024".to_owned(),
                        resp_json_err,
                        resp_json_message,
                    ),
                    _ => ApiResponseError::InternalServerError(
                        "E0024".to_owned(),
                        resp_json_err,
                        resp_json_message,
                    ),
                };

                Err(mapped_res_error.into())
            }
        };
    }

    pub fn get<T>(&self, endpoint: &str) -> anyhow::Result<T>
    where
        T: DeserializeOwned,
    {
        let response = self.base_api_client.get(endpoint);
        let response_handled: T = self.handle_response::<T>(response)?;

        Ok(response_handled)
    }

    pub fn new() -> Self {
        let ac = ApiClient {
            base_url: EnvValues::API_BASE_URL.to_owned(),
            connect_timeout_ms: DefaultValues::API_TIMEOUT,
            enable_compression: true,
        };

        Self {
            base_api_client: ac,
        }
    }
}
