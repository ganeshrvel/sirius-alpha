use crate::constants::environment::APP_ENV;
use crate::EnvValues;
use attohttpc::body::{Body, Bytes};
use attohttpc::header::{HeaderValue, IntoHeaderName};
use attohttpc::{Error, Method, RequestBuilder, Response};
use log::{debug, info, warn};
use std::any::Any;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt::Debug;
use std::time::Duration;

pub mod sirius_proxima;

pub struct ApiClient {
    pub base_url: String,
    pub connect_timeout_ms: u64,
    pub enable_compression: bool,
}

impl ApiClient {
    fn build_url(&self, endpoint: &str) -> String {
        format!("{}{}", EnvValues::API_BASE_URL, endpoint)
    }

    fn req_builder<H, V>(
        &self,
        method: Method,
        endpoint: &str,
        headers: Option<HashMap<H, V>>,
        query_params: Option<HashMap<&str, &str>>,
    ) -> RequestBuilder
        where
            H: IntoHeaderName + Debug,
            V: TryInto<HeaderValue> + Debug,
            Error: From<V::Error>,
    {
        let api_url = self.build_url(endpoint);

        let mut req = RequestBuilder::new(method, api_url.as_str());
        if let Some(headers_ok) = headers {
            for (h, v) in headers_ok {
                req = req.header(h, v);
            }
        }

        if let Some(query_params_ok) = query_params {
            for (k, v) in query_params_ok {
                req = req.param(k, v);
            }
        }

        let req = req.danger_accept_invalid_certs(true);
        let req = req.danger_accept_invalid_hostnames(true);
        let req = req.connect_timeout(Duration::from_millis(self.connect_timeout_ms));
        let req = req.allow_compression(self.enable_compression);
        req.connect_timeout(Duration::from_millis(15000))
    }

    fn log_request<B>(&self, mut req: RequestBuilder<B>) -> RequestBuilder<B>
        where
            B: Body + Debug + Any + Clone,
    {
        if APP_ENV.config.show_network_requests {
            debug!("\n\n");
            debug!("=======================================");
            warn!("Request inspector");

            let mut req_i = req.inspect();
            let req_body = req_i.body();

            let req_body_any = &req_body.clone() as &dyn Any;

            if let Some(req_body_buf) = req_body_any.downcast_ref::<Bytes<Vec<u8>>>() {
                if let Ok(s) = std::str::from_utf8(&*req_body_buf.0) {
                    info!("Body: {}", s);
                }
            }

            let req_url = req_i.url();
            let req_headers = req_i.headers();
            let req_method = req_i.method();

            info!("Url: {}", req_url.as_str());
            info!("Method: {}", req_method.as_str());
            info!("Header: {:?}", req_headers);

            debug!("=======================================\n\n");
        }

        req
    }

    fn log_response(&self, res: &Response) {
        if APP_ENV.config.show_network_response {
            debug!("\n\n");
            debug!("=======================================");
            warn!("Response inspector");

            let headers = res.headers();
            let status = res.status();
            let is_success = res.is_success();

            info!("Status: {}", status);
            info!("Is success: {}", is_success);
            info!("Headers: {:?}", headers);

            debug!("=======================================\n\n");
        }
    }

    fn process_request<B>(&self, req: RequestBuilder<B>) -> attohttpc::Result<Response>
        where
            B: Body + Debug + Any + Clone,
    {
        let req = self.log_request(req);
        let res = req.send();

        match res {
            Ok(r) => {
                self.log_response(&r);

                Ok(r)
            }
            Err(_) => res,
        }
    }

    fn get<H, V>(
        &self,
        endpoint: &str,
        headers: Option<HashMap<H, V>>,
        query_params: Option<HashMap<&str, &str>>,
    ) -> attohttpc::Result<Response>
        where
            H: IntoHeaderName + Debug,
            V: TryInto<HeaderValue> + Debug,
            Error: From<V::Error>,
    {
        let req = self.req_builder(Method::GET, endpoint, headers, query_params);

        self.process_request(req)
    }

    fn put<Se, H, V>(
        &self,
        endpoint: &str,
        body: &Se,
        headers: Option<HashMap<H, V>>,
        query_params: Option<HashMap<&str, &str>>,
    ) -> attohttpc::Result<Response>
        where
            Se: serde::Serialize,
            H: IntoHeaderName + Debug,
            V: TryInto<HeaderValue> + Debug,

            Error: From<V::Error>,
    {
        let req = self.req_builder(Method::PUT, endpoint, headers, query_params);
        let req = req.json(body)?;

        self.process_request(req)
    }
}
