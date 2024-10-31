use thiserror::Error;

use crate::frame::ProtocolError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Protocol error occurred: {0:?}")]
    Protocol(#[from] ProtocolError), // 将 ProtocolError 包装为 Protocol 错误
    #[error(transparent)]
    Transport(#[from] std::io::Error),
}
