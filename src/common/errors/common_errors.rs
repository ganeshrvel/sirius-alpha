use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommonError {
    #[error("[0:?] A Mutex Guard error occurred {1:?}")]
    MutexGuard(String, String),
}
