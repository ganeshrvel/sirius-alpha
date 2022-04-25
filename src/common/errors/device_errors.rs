use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeviceError<'a> {
    #[error("[0:?] A peripheral pin error occurred: {1:?}")]
    PeripheralPin(&'a str, &'a str),
}
