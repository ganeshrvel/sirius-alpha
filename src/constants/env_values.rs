use std::marker::PhantomData;

pub struct EnvValues<'a> {
    /// https://stackoverflow.com/questions/40484154/parameter-a-is-never-used-error-when-a-is-used-in-type-parameter-bound
    // Causes the type to function *as though* it has a `&'a ()` field,
    // despite not *actually* having one.
    _marker: PhantomData<&'a ()>,
}

impl EnvValues<'static> {
    pub const WIFI_SSID: &'static str = dotenv!("WIFI_SSID");

    pub const WIFI_PASS: &'static str = dotenv!("WIFI_PASS");

    pub const API_TOKEN_KEY: &'static str = dotenv!("API_TOKEN_KEY");

    pub const API_SECRET_TOKEN: &'static str = dotenv!("API_SECRET_TOKEN");

    pub const API_BASE_URL: &'static str = dotenv!("API_BASE_URL");

    pub const DEVICE_TYPE: &'static str = dotenv!("DEVICE_TYPE");

    pub const DEVICE_NAME: &'static str = dotenv!("DEVICE_NAME");

    pub const DEVICE_ID: &'static str = dotenv!("DEVICE_ID");

    pub const DEVICE_LOCATION: &'static str = dotenv!("DEVICE_LOCATION");

    pub const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");
}
