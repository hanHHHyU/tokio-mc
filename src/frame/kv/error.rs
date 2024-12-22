use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum KVError {
    #[error("Invalid number format: {input}. Error: {source}")]
    InvalidNumberFormat {
        input: String,
        source: std::num::ParseIntError,
    },

    #[error("Hexadecimal parsing failed for: {0}")]
    HexParseError(String),

    // 基恩士PLC错误
    #[error("Keyence PLC address invalid")]
    AddressInvalid,

    #[error("Keyence PLC convert error")]
    ConvertError,

    #[error("Keyence PLC map not found")]
    MapNotFound,

    #[error("Keyence PLC parse error")]
    PaseError,

    #[error("Keyence PLC address not found")]
    AddressNotFound,

    #[error("Parse number error")]
    ParseNumberError,

    #[error("Unknown error occurred: {0}")]
    Unknown(String),
}

// 实现 `From<std::num::ParseIntError>`，便于错误转换
impl From<std::num::ParseIntError> for KVError {
    fn from(err: std::num::ParseIntError) -> Self {
        KVError::InvalidNumberFormat {
            input: String::new(),
            source: err,
        }
    }
}
