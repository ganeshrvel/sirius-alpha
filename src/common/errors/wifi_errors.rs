use thiserror::Error;

#[derive(Error, Debug)]
pub enum WifiError {
    #[error("[0:?] a wifi scanning error occurred {1:?}")]
    Scanning(String, String),

    #[error("[0:?] the wifi access point `{1:?}` was NOT found")]
    ApNotFound(String, String),

    #[error("[0:?] a wifi configuration error occurred {1:?}")]
    Configuration(String, String),
}
