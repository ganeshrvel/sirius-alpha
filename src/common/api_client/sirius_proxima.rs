use crate::common::api_client::ApiClient;
use crate::common::errors::api_errors::{ApiClientError, ApiResponseError};
use crate::common::models::sirius_proxima_api::{
    SiriusProximaErrorResponse, SiriusProximaSuccessResponse,
};
use crate::constants::default_values::DefaultValues;
use crate::EnvValues;
use attohttpc::header::{HeaderValue, IntoHeaderName};
use attohttpc::{Error, ErrorKind, Response, StatusCode};
use lazy_static::lazy_static;
use log::error;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt::Debug;

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

pub type ApiResponse<De: DeserializeOwned> = anyhow::Result<De>;

impl SiriusProximaClient {
    fn handle_response<De: DeserializeOwned>(
        &self,
        response: attohttpc::Result<Response>,
    ) -> anyhow::Result<De> {
        let resp_ok = match response {
            Ok(server_response) => {
                let is_success = server_response.is_success();
                let status_code = server_response.status();
                let resp_text = server_response.text();
                let resp_text_ok = match resp_text {
                    Ok(s) => s,
                    Err(e) => {
                        error!("[E0021a][SiriusProximaClient] {}", e.to_string());

                        return Err(
                            ApiClientError::Response("E0021b".to_owned(), e.to_string()).into()
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
                error!("[E0020a][SiriusProximaClient] {}", e.to_string());

                let err_str = e.to_string();
                let err_kind = e.into_kind();

                return match err_kind {
                    ErrorKind::Io(io_err) => Err(ApiResponseError::SiteNotFound(
                        "E0025".to_owned(),
                        io_err.to_string(),
                    )
                    .into()),

                    _ => Err(ApiClientError::Response("E0020b".to_owned(), err_str).into()),
                };
            }
        };

        return match resp_ok {
            ResponseType::OkResponse(d, _) => {
                let resp_json = serde_json::from_str::<SiriusProximaSuccessResponse<De>>(&*d);
                let resp_json_ok = match resp_json {
                    Ok(s) => s,
                    Err(e) => {
                        error!("[E0022a][SiriusProximaClient] {}", e.to_string());

                        return Err(ApiClientError::JsonParsing(
                            "E0022b".to_owned(),
                            e.to_string(),
                        )
                        .into());
                    }
                };

                Ok(resp_json_ok.data)
            }
            ResponseType::ErrorResponse(d, status_code) => {
                let resp_json = serde_json::from_str::<SiriusProximaErrorResponse>(&*d);
                let resp_json_ok = match resp_json {
                    Ok(s) => s,
                    Err(e) => {
                        error!("[E0023a][SiriusProximaClient] {}", e.to_string());

                        return Err(ApiClientError::JsonParsing(
                            "E0023b".to_owned(),
                            e.to_string(),
                        )
                        .into());
                    }
                };

                let resp_json_err = resp_json_ok.error.unwrap_or_default();
                let resp_json_message = resp_json_ok.message.unwrap_or_default();

                error!("[E0024a][SiriusProximaClient] Received an API response error. Error: {}, Message: {}", resp_json_err,resp_json_message );
                let mapped_res_error = match status_code {
                    StatusCode::BAD_REQUEST => ApiResponseError::BadRequest(
                        "E0024b".to_owned(),
                        resp_json_err,
                        resp_json_message,
                    ),
                    StatusCode::NOT_FOUND => ApiResponseError::NotFound(
                        "E0024c".to_owned(),
                        resp_json_err,
                        resp_json_message,
                    ),
                    _ => ApiResponseError::InternalServerError(
                        "E0024d".to_owned(),
                        resp_json_err,
                        resp_json_message,
                    ),
                };

                Err(mapped_res_error.into())
            }
        };
    }

    pub fn get<De, H, V>(
        &self,
        endpoint: &str,
        headers: Option<HashMap<H, V>>,
        query_params: Option<HashMap<&str, &str>>,
    ) -> ApiResponse<De>
    where
        De: DeserializeOwned,
        H: IntoHeaderName + Debug,
        V: TryInto<HeaderValue> + Debug,
        Error: From<V::Error>,
    {
        let response = self.base_api_client.get(endpoint, headers, query_params);
        let response_handled: De = self.handle_response::<De>(response)?;

        Ok(response_handled)
    }

    /// Put Method:
    ///
    ///
    /// Headers:
    /// # Example
    /// ```
    /// let mut headers = HashMap::new();
    ///     headers.insert(
    ///         "header-key","header-value"
    ///     );
    /// ```
    ///
    /// Query String params:
    /// # Example
    /// ```
    ///  let mut query_params = HashMap::new();
    //   query_params.insert("param-1", "value-1");
    /// ```
    pub fn put<De, Se, H, V>(
        &self,
        endpoint: &str,
        body: &Se,
        headers: Option<HashMap<H, V>>,
        query_params: Option<HashMap<&str, &str>>,
    ) -> ApiResponse<De>
    where
        De: DeserializeOwned,
        Se: serde::Serialize,
        H: IntoHeaderName + Debug,
        V: TryInto<HeaderValue> + Debug,

        Error: From<V::Error>,
    {
        let response = self
            .base_api_client
            .put::<Se, H, V>(endpoint, body, headers, query_params);
        let response_handled: De = self.handle_response::<De>(response)?;

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

#[derive(Debug, Deserialize)]
pub struct PingResponse {
    pub short_period_buzzer_beep_duration_ms: usize,
    pub is_continuous_period_buzzer_beep_active: bool,
}
