use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("The number of points to read or write is out of the allowed range.")]
    OutOfRange,

    #[error("The function code received is not recognized or is invalid: {0:?}")]
    InvalidFunctionCode([u8; 4]),

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("This functionality is not yet implemented.")]
    NotImplemented,
}

pub fn map_error_code(error_code: u16) -> Option<ProtocolError> {
    match error_code {
        0xC051..=0xC054 => Some(ProtocolError::OutOfRange),
        // 其他错误映射
        _ => None,
    }
}
