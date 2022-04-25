use thiserror::Error;

#[derive(Error, Debug)]
pub enum SetupError<'a> {
    #[error("[0:?] A peripherals error occurred: {1:?}")]
    Peripherals(&'a str, &'a str),
}
