use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("The number of points to read or write is out of the allowed range.")]
    OutOfRange,
}
