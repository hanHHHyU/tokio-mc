use std::{
    borrow::Cow,
    error,
    fmt::{self, Display},
};

use crate::bytes::BytesMut;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionCode {
    ReadBits,
    ReadWords,
    WriteMultipleBits,
    WriteMultipleWords,
}

impl FunctionCode {
    /// Create a new [`FunctionCode`] with `value`.
    #[must_use]
    pub fn new(value: BytesMut) -> Option<Self> {
        match &value[..] {
            [0x01, 0x04, 0x01, 0x00] => Some(Self::ReadBits), // 假设这对应 ReadBits
            [0x01, 0x04, 0x00, 0x00] => Some(Self::ReadWords), // 假设这对应 ReadWords
            [0x01, 0x14, 0x01, 0x00] => Some(Self::WriteMultipleBits), // 对应 WriteMultipleBits
            [0x01, 0x14, 0x00, 0x00] => Some(Self::WriteMultipleWords), // 对应 WriteMultipleWords
            _ => None,                                        // 如果字节序列不匹配，返回 None
        }
    }

    /// 将 `FunctionCode` 转换为相应的 `BytesMut` 字节序列
    #[must_use]
    pub fn value(self) -> BytesMut {
        let mut buf = BytesMut::new();
        match self {
            FunctionCode::ReadBits => {
                buf.extend_from_slice(&[0x01, 0x04, 0x01, 0x00]); // 对应 ReadBits 的字节序列
            }
            FunctionCode::ReadWords => {
                buf.extend_from_slice(&[0x01, 0x04, 0x00, 0x00]); // 对应 ReadWords 的字节序列
            }
            FunctionCode::WriteMultipleBits => {
                buf.extend_from_slice(&[0x01, 0x14, 0x01, 0x00]); // 对应 WriteMultipleBits 的字节序列
            }
            FunctionCode::WriteMultipleWords => {
                buf.extend_from_slice(&[0x01, 0x14, 0x00, 0x00]); // 对应 WriteMultipleWords 的字节序列
            }
        }
        buf
    }
}

impl Display for FunctionCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes = self.value(); // 获取 BytesMut
        write!(f, "{:?}", &bytes[..]) // 使用 Debug 格式化字节切片
    }
}

pub(crate) mod tcp;

pub type Address = u32;

pub(crate) type Bit = bool;

pub(crate) type Word = u16;

pub type Quantity = u16;

pub(crate) const REQUEST_BYTE_LAST_LEN: usize = 6;

// 软元件代码的枚举
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SoftElementCode {
    X = 0x9C, // 输入继电器
    Y = 0x9D, // 输出继电器
    D = 0xA8, // 数据寄存器
    M = 0x90, // 内存继电器
              // 其他软元件代码可以继续添加
}

// 请求的枚举，类似你给出的Modbus请求设计
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request<'a> {
    /// 读取多个软元件（位），如读取继电器或输入
    ReadBits(Address, Quantity, SoftElementCode),

    /// 读取多个软元件（字），如数据寄存器
    ReadWords(Address, Quantity, SoftElementCode),

    /// 写入多个位，如继电器或输入
    WriteMultipleBits(Address, Cow<'a, [Bit]>, SoftElementCode),

    /// 写入多个字，如数据寄存器
    WriteMultipleWords(Address, Cow<'a, [Word]>, SoftElementCode),
}

// 实现辅助功能，比如将请求转换为'owned'版本或获取功能码
impl<'a> Request<'a> {
    /// 将请求转换为'owned'的实例（静态生命周期）
    #[must_use]
    pub fn into_owned(self) -> Request<'static> {
        use Request::*;
        match self {
            ReadBits(addr, qty, code) => ReadBits(addr, qty, code),
            ReadWords(addr, qty, code) => ReadWords(addr, qty, code),
            WriteMultipleBits(addr, coils, code) => {
                WriteMultipleBits(addr, Cow::Owned(coils.into_owned()), code)
            }
            WriteMultipleWords(addr, words, code) => {
                WriteMultipleWords(addr, Cow::Owned(words.into_owned()), code)
            }
        }
    }

    #[must_use]
    pub const fn function_code(&self) -> FunctionCode {
        use Request::*;
        match self {
            ReadBits(_, _, _) => FunctionCode::ReadBits,
            ReadWords(_, _, _) => FunctionCode::ReadWords,
            WriteMultipleBits(_, _, _) => FunctionCode::WriteMultipleBits,
            WriteMultipleWords(_, _, _) => FunctionCode::WriteMultipleWords,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Response {
    ReadBits(Vec<Bit>),
    ReadWords(Vec<Word>),
    WriteMultipleBits(Address, Quantity, SoftElementCode),
    WriteMultipleWords(Address, Quantity, SoftElementCode),
}

impl Response {
    #[must_use]
    pub const fn function_code(&self) -> FunctionCode {
        use Response::*;

        match self {
            ReadBits(_) => FunctionCode::ReadBits,
            ReadWords(_) => FunctionCode::ReadWords,
            WriteMultipleBits(_, _, _) => FunctionCode::WriteMultipleBits,
            WriteMultipleWords(_, _, _) => FunctionCode::WriteMultipleWords,
        }
    }
}

/// A server (slave) exception.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExceptionCode {
    /// 0x01
    IllegalFunction,
    /// 0x02
    IllegalDataAddress,
    /// 0x03
    IllegalDataValue,
    /// 0x04
    ServerDeviceFailure,
    /// 0x05
    Acknowledge,
    /// 0x06
    ServerDeviceBusy,
    /// 0x08
    MemoryParityError,
    /// 0x0A
    GatewayPathUnavailable,
    /// 0x0B
    GatewayTargetDevice,
    /// None of the above.
    ///
    /// Although encoding one of the predefined values as this is possible, it is not recommended.
    /// Instead, prefer to use [`Self::new()`] to prevent such ambiguities.
    Custom(u8),
}

impl From<ExceptionCode> for u8 {
    fn from(from: ExceptionCode) -> Self {
        use crate::frame::ExceptionCode::*;
        match from {
            IllegalFunction => 0x01,
            IllegalDataAddress => 0x02,
            IllegalDataValue => 0x03,
            ServerDeviceFailure => 0x04,
            Acknowledge => 0x05,
            ServerDeviceBusy => 0x06,
            MemoryParityError => 0x08,
            GatewayPathUnavailable => 0x0A,
            GatewayTargetDevice => 0x0B,
            Custom(code) => code,
        }
    }
}

impl ExceptionCode {
    /// Create a new [`ExceptionCode`] with `value`.
    #[must_use]
    pub const fn new(value: u8) -> Self {
        use crate::frame::ExceptionCode::*;

        match value {
            0x01 => IllegalFunction,
            0x02 => IllegalDataAddress,
            0x03 => IllegalDataValue,
            0x04 => ServerDeviceFailure,
            0x05 => Acknowledge,
            0x06 => ServerDeviceBusy,
            0x08 => MemoryParityError,
            0x0A => GatewayPathUnavailable,
            0x0B => GatewayTargetDevice,
            other => Custom(other),
        }
    }

    pub(crate) fn description(&self) -> &str {
        use crate::frame::ExceptionCode::*;

        match *self {
            IllegalFunction => "Illegal function",
            IllegalDataAddress => "Illegal data address",
            IllegalDataValue => "Illegal data value",
            ServerDeviceFailure => "Server device failure",
            Acknowledge => "Acknowledge",
            ServerDeviceBusy => "Server device busy",
            MemoryParityError => "Memory parity error",
            GatewayPathUnavailable => "Gateway path unavailable",
            GatewayTargetDevice => "Gateway target device failed to respond",
            Custom(_) => "Custom",
        }
    }
}

/// A server (slave) exception response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExceptionResponse {
    pub function: FunctionCode,
    pub exception: ExceptionCode,
}

/// Represents a message from the client (slave) to the server (master).
#[derive(Debug, Clone)]
pub(crate) struct RequestPdu<'a>(pub(crate) Request<'a>);

impl<'a> From<Request<'a>> for RequestPdu<'a> {
    fn from(from: Request<'a>) -> Self {
        RequestPdu(from)
    }
}

impl<'a> From<RequestPdu<'a>> for Request<'a> {
    fn from(from: RequestPdu<'a>) -> Self {
        from.0
    }
}

/// Represents a message from the server (slave) to the client (master).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResponsePdu(pub(crate) Result<Response, ExceptionResponse>);

impl From<Response> for ResponsePdu {
    fn from(from: Response) -> Self {
        ResponsePdu(Ok(from))
    }
}

impl From<ExceptionResponse> for ResponsePdu {
    fn from(from: ExceptionResponse) -> Self {
        ResponsePdu(Err(from))
    }
}

impl From<ResponsePdu> for Result<Response, ExceptionResponse> {
    fn from(from: ResponsePdu) -> Self {
        from.0
    }
}

impl fmt::Display for ExceptionCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl error::Error for ExceptionCode {
    fn description(&self) -> &str {
        self.description()
    }
}

impl fmt::Display for ExceptionResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Modbus function {}: {}", self.function, self.exception)
    }
}

impl error::Error for ExceptionResponse {
    fn description(&self) -> &str {
        self.exception.description()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_function_code() {
        assert_eq!(
            FunctionCode::ReadBits,
            FunctionCode::new(BytesMut::from(&[0x01, 0x04, 0x01, 0x00][..]))
                .expect("Failed to create FunctionCode from bytes")
        );
        assert_eq!(
            FunctionCode::ReadWords,
            FunctionCode::new(BytesMut::from(&[0x01, 0x04, 0x00, 0x00][..]))
                .expect("Failed to create FunctionCode from bytes")
        );

        assert_eq!(
            FunctionCode::WriteMultipleBits,
            FunctionCode::new(BytesMut::from(&[0x01, 0x14, 0x01, 0x00][..]))
                .expect("Failed to create FunctionCode from bytes")
        );

        assert_eq!(
            FunctionCode::WriteMultipleWords,
            FunctionCode::new(BytesMut::from(&[0x01, 0x14, 0x00, 0x00][..]))
                .expect("Failed to create FunctionCode from bytes")
        );
    }

    #[test]
    fn function_code_values() {
        let read_bits_bytes = BytesMut::from(&[0x01, 0x04, 0x01, 0x00][..]);
        let read_words_bytes = BytesMut::from(&[0x01, 0x04, 0x00, 0x00][..]);
        let write_multiple_bits_bytes = BytesMut::from(&[0x01, 0x14, 0x01, 0x00][..]);
        let write_multiple_words_bytes = BytesMut::from(&[0x01, 0x14, 0x00, 0x00][..]);

        // ReadBits 测试
        assert_eq!(
            FunctionCode::ReadBits.value(),
            read_bits_bytes,
            "ReadBits byte sequence is incorrect"
        );

        // ReadWords 测试
        assert_eq!(
            FunctionCode::ReadWords.value(),
            read_words_bytes,
            "ReadWords byte sequence is incorrect"
        );

        // WriteMultipleBits 测试
        assert_eq!(
            FunctionCode::WriteMultipleBits.value(),
            write_multiple_bits_bytes,
            "WriteMultipleBits byte sequence is incorrect"
        );

        // WriteMultipleWords 测试
        assert_eq!(
            FunctionCode::WriteMultipleWords.value(),
            write_multiple_words_bytes,
            "WriteMultipleWords byte sequence is incorrect"
        );
    }
}
