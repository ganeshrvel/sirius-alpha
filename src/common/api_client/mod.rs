use crate::constants::default_values::DefaultValues;
use crate::EnvValues;
use attohttpc::{body, Method, RequestBuilder, Response};
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

    fn req_builder(&self, method: Method, endpoint: &str) -> RequestBuilder {
        let api_url = self.build_url(endpoint);

        let req = attohttpc::RequestBuilder::new(method, api_url.as_str());
        let req = req.danger_accept_invalid_certs(true);
        let req = req.danger_accept_invalid_hostnames(true);
        let req = req.connect_timeout(Duration::from_millis(self.connect_timeout_ms));
        req.allow_compression(self.enable_compression)
    }

    fn get(&self, endpoint: &str) -> attohttpc::Result<Response> {
        let req = self.req_builder(Method::GET, endpoint);
        req.send()
    }

    // fn put(&self, endpoint: &str,  body: body::Json<>) -> attohttpc::Result<Response> {
    //     let req = self.req_builder(Method::PUT, endpoint);
    //     req.body();
    //     req.send()
    // }
}
