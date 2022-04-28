use thiserror::Error;

#[derive(Error, Debug)]
pub enum TmError<E> {
    #[error("an ack error occured")]
    Ack,

    #[error("an IO error occured")]
    IO(E),

    #[error("[0:?] an auto scroll error occured: {1:?}")]
    AutoScroll(String, String),
}

impl<E> From<E> for TmError<E> {
    fn from(err: E) -> Self {
        Self::IO(err)
    }
}
