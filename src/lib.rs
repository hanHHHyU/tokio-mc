pub use self::frame::{
    Address, ExceptionCode, ExceptionResponse, FunctionCode, Quantity, Request, Response,
};
pub use bytes;

mod error;
pub use self::error::{Error, ProtocolError};

mod frame;

mod codec;

mod client;

mod header;

/// Specialized [`std::result::Result`] type for type-checked responses of the _Modbus_ client API.
///
/// The payload is generic over the response type.
///
/// This [`Result`] type contains 2 layers of errors.
///
/// 1. [`Error`]: An unexpected protocol or network error that occurred during client/server communication.
/// 2. [`ExceptionCode`]: An error occurred on the _Modbus_ server.
pub type Result<T> = std::result::Result<std::result::Result<T, ExceptionCode>, Error>;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
