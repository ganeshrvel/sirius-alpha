use serde::{Deserialize, Serialize};

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
