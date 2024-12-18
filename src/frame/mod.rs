use std::{
    borrow::Cow,
    fmt::{self, Display},
};

pub use types::*;

use crate::bytes::BytesMut;

mod error;
mod kv;
mod map;
mod regex;
mod types;

pub use error::{map_error_code, ProtocolError};

pub use map::{convert_to_base, find_instruction_code};
pub use regex::split_address;

pub use kv::convert_keyence_to_mitsubishi_address;

pub use kv::KVError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionCode {
    ReadBools,
    ReadU16s,
    WriteBools,
    WriteU16s,
}

impl FunctionCode {
    /// Create a new [`FunctionCode`] with `value`.
    #[must_use]
    pub fn new(value: BytesMut) -> Option<Self> {
        match &value[..] {
            [0x01, 0x04, 0x01, 0x00] => Some(Self::ReadBools), // 假设这对应 ReadBits
            [0x01, 0x04, 0x00, 0x00] => Some(Self::ReadU16s),  // 假设这对应 ReadWords
            [0x01, 0x14, 0x01, 0x00] => Some(Self::WriteBools), // 对应 WriteMultipleBits
            [0x01, 0x14, 0x00, 0x00] => Some(Self::WriteU16s), // 对应 WriteMultipleWords
            _ => None,                                         // 如果字节序列不匹配，返回 None
        }
    }

    /// 将 `FunctionCode` 转换为相应的 `BytesMut` 字节序列
    #[must_use]
    pub fn value(self) -> BytesMut {
        let mut buf = BytesMut::new();
        match self {
            FunctionCode::ReadBools => {
                buf.extend_from_slice(&[0x01, 0x04, 0x00, 0x00]); // 对应 ReadBits 的字节序列
            }
            FunctionCode::ReadU16s => {
                buf.extend_from_slice(&[0x01, 0x04, 0x00, 0x00]); // 对应 ReadWords 的字节序列
            }
            FunctionCode::WriteBools => {
                buf.extend_from_slice(&[0x01, 0x14, 0x01, 0x00]); // 对应 WriteMultipleBits 的字节序列
            }
            FunctionCode::WriteU16s => {
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

// 请求的枚举，类似你给出的Modbus请求设计
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request<'a> {
    ReadBools(Cow<'a, str>, Quantity),
    ReadU16s(Cow<'a, str>, Quantity),
    WriteBools(Cow<'a, str>, Cow<'a, [bool]>),
    WriteU16s(Cow<'a, str>, Cow<'a, [u16]>),
}

// 实现辅助功能，比如将请求转换为'owned'版本或获取功能码
impl<'a> Request<'a> {
    /// 将请求转换为'owned'的实例（静态生命周期）
    #[must_use]
    pub fn into_owned(self) -> Request<'static> {
        use Request::*;
        match self {
            ReadBools(addr, qty) => ReadBools(Cow::Owned(addr.into_owned()), qty),
            ReadU16s(addr, qty) => ReadU16s(Cow::Owned(addr.into_owned()), qty),
            WriteBools(addr, coils) => WriteBools(
                Cow::Owned(addr.into_owned()),
                Cow::Owned(coils.into_owned()),
            ),
            WriteU16s(addr, words) => WriteU16s(
                Cow::Owned(addr.into_owned()),
                Cow::Owned(words.into_owned()),
            ),
        }
    }

    #[must_use]
    pub const fn function_code(&self) -> FunctionCode {
        use Request::*;
        match self {
            ReadBools(_, _) => FunctionCode::ReadBools,
            ReadU16s(_, _) => FunctionCode::ReadU16s,
            WriteBools(_, _) => FunctionCode::WriteBools,
            WriteU16s(_, _) => FunctionCode::WriteU16s,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Response {
    ReadBools(Vec<bool>),
    ReadU16s(Vec<u16>),
    // WriteMultipleBits(Address, Quantity, SoftElementCode),
    // WriteMultipleWords(Address, Quantity, SoftElementCode),
    WriteBools(),
    WriteU16s(),
}

impl Response {
    #[must_use]
    pub const fn function_code(&self) -> FunctionCode {
        use Response::*;

        match self {
            ReadBools(_) => FunctionCode::ReadBools,
            ReadU16s(_) => FunctionCode::ReadU16s,
            // WriteMultipleBits(_, _, _) => FunctionCode::WriteMultipleBits,
            // WriteMultipleWords(_, _, _) => FunctionCode::WriteMultipleWords,
            WriteBools() => FunctionCode::WriteBools,
            WriteU16s() => FunctionCode::WriteU16s,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_function_code() {
        assert_eq!(
            FunctionCode::ReadBools,
            FunctionCode::new(BytesMut::from(&[0x01, 0x04, 0x01, 0x00][..]))
                .expect("Failed to create FunctionCode from bytes")
        );
        assert_eq!(
            FunctionCode::ReadU16s,
            FunctionCode::new(BytesMut::from(&[0x01, 0x04, 0x00, 0x00][..]))
                .expect("Failed to create FunctionCode from bytes")
        );

        assert_eq!(
            FunctionCode::WriteBools,
            FunctionCode::new(BytesMut::from(&[0x01, 0x14, 0x01, 0x00][..]))
                .expect("Failed to create FunctionCode from bytes")
        );

        assert_eq!(
            FunctionCode::WriteU16s,
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
            FunctionCode::ReadBools.value(),
            read_bits_bytes,
            "ReadBits byte sequence is incorrect"
        );

        // ReadWords 测试
        assert_eq!(
            FunctionCode::ReadU16s.value(),
            read_words_bytes,
            "ReadWords byte sequence is incorrect"
        );

        // WriteMultipleBits 测试
        assert_eq!(
            FunctionCode::WriteBools.value(),
            write_multiple_bits_bytes,
            "WriteMultipleBits byte sequence is incorrect"
        );

        // WriteMultipleWords 测试
        assert_eq!(
            FunctionCode::WriteU16s.value(),
            write_multiple_words_bytes,
            "WriteMultipleWords byte sequence is incorrect"
        );
    }
}
