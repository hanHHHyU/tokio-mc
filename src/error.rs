use thiserror::Error;

use crate::frame::{KVError, ProtocolError};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Protocol error occurred: {0:?}")]
    Protocol(#[from] ProtocolError), // 将 ProtocolError 包装为 Protocol 错误

    #[error(transparent)]
    Transport(#[from] std::io::Error),

    #[error("Keyence-specific error: {0}")]
    KV(#[from] KVError),

    #[error("Utf8 error: {0}")]
    Utf8Error(String),
}
