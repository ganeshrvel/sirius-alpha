use crate::helpers::chip_info::{ChipInfo, Model};
use crate::EnvValues;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum_macros::EnumString;

#[derive(Debug, Deserialize, Serialize)]
pub struct Health {
    pub is_health_ok: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SiriusProximaSuccessResponse<T> {
    pub status_code: u16,
    pub message: Option<String>,
    pub data: T,
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SiriusProximaErrorResponse {
    pub status_code: u16,
    pub message: Option<String>,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SiriusProximaPing {
    pub device_type: DeviceType,
    pub device: Device,
}

impl SiriusProximaPing {
    pub fn new(is_first_ping_after_device_turned_on: bool) -> anyhow::Result<Self> {
        Ok(Self {
            device_type: DeviceType::from_str(EnvValues::DEVICE_TYPE)?,
            device: Device::new(is_first_ping_after_device_turned_on)?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Device {
    pub device_type: DeviceType,
    pub details: DeviceDetails,
}

impl Device {
    pub fn new(is_first_ping_after_device_turned_on: bool) -> anyhow::Result<Self> {
        Ok(Self {
            device_type: DeviceType::from_str(EnvValues::DEVICE_TYPE)?,
            details: DeviceDetails::new(is_first_ping_after_device_turned_on),
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, EnumString)]
pub enum DeviceType {
    #[strum(serialize = "roof_water_heater")]
    #[serde(rename = "roof_water_heater")]
    RoofWaterHeater,

    #[strum(serialize = "bore_well_motor")]
    #[serde(rename = "bore_well_motor")]
    BoreWellMotor,

    #[strum(serialize = "ground_well_motor")]
    #[serde(rename = "ground_well_motor")]
    GroundWellMotor,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceDetails {
    pub device_name: String,
    pub model: Model,
    pub device_id: String,
    pub device_location: String,
    pub revision: u8,
    pub app_version: String,
    pub is_first_ping_after_device_turned_on: bool,
}

impl DeviceDetails {
    pub fn new(is_first_ping_after_device_turned_on: bool) -> Self {
        let chip = ChipInfo::new();

        Self {
            device_name: EnvValues::DEVICE_NAME.to_owned(),
            device_id: EnvValues::DEVICE_ID.to_owned(),
            device_location: EnvValues::DEVICE_LOCATION.to_owned(),
            app_version: EnvValues::APP_VERSION.to_owned(),
            model: chip.model.unwrap_or(Model::Unknown),
            revision: chip.revision,
            is_first_ping_after_device_turned_on,
        }
    }
}
