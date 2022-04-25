use thiserror::Error;

#[derive(Error, Debug)]
pub enum WifiError {
    #[error("[0:?] a wifi scanning error occurred {1:?}")]
    Scanning(String, String),

    #[error("[0:?] the wifi access point `{1:?}` was NOT found")]
    ApNotFound(String, String),

    #[error("[0:?] a wifi configuration error occurred {1:?}")]
    Configuration(String, String),

    #[error("[0:?] network connection not available")]
    NotConnected(String),

    #[error("[0:?] unable to resolve a network IP address")]
    UnresolvedIp(String),
}
