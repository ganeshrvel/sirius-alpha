use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SAlphaDeviceDetails {
    pub device_name: String,
    pub model: String,
    pub device_id: String,
    pub device_location: String,
    pub device_sdk: String,
    pub app_version: String,
}
