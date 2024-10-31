use std::{
    borrow::Cow,
    fmt::{self, Display},
};

pub use types::*;

use crate::bytes::BytesMut;

mod error;
mod map;
mod regex;
mod types;

pub use error::{map_error_code, ProtocolError};

pub use map::{convert_to_base, find_instruction_code};
pub use regex::split_address;

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

// 请求的枚举，类似你给出的Modbus请求设计
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request<'a> {
    ReadBits(Cow<'a, str>, Quantity),
    ReadWords(Cow<'a, str>, Quantity),
    WriteMultipleBits(Cow<'a, str>, Cow<'a, [Bit]>),
    WriteMultipleWords(Cow<'a, str>, Cow<'a, [Word]>),
}

// 实现辅助功能，比如将请求转换为'owned'版本或获取功能码
impl<'a> Request<'a> {
    /// 将请求转换为'owned'的实例（静态生命周期）
    #[must_use]
    pub fn into_owned(self) -> Request<'static> {
        use Request::*;
        match self {
            ReadBits(addr, qty) => ReadBits(Cow::Owned(addr.into_owned()), qty),
            ReadWords(addr, qty) => ReadWords(Cow::Owned(addr.into_owned()), qty),
            WriteMultipleBits(addr, coils) => WriteMultipleBits(
                Cow::Owned(addr.into_owned()),
                Cow::Owned(coils.into_owned()),
            ),
            WriteMultipleWords(addr, words) => WriteMultipleWords(
                Cow::Owned(addr.into_owned()),
                Cow::Owned(words.into_owned()),
            ),
        }
    }

    #[must_use]
    pub const fn function_code(&self) -> FunctionCode {
        use Request::*;
        match self {
            ReadBits(_, _) => FunctionCode::ReadBits,
            ReadWords(_, _) => FunctionCode::ReadWords,
            WriteMultipleBits(_, _) => FunctionCode::WriteMultipleBits,
            WriteMultipleWords(_, _) => FunctionCode::WriteMultipleWords,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Response {
    ReadBits(Vec<Bit>),
    ReadWords(Vec<Word>),
    // WriteMultipleBits(Address, Quantity, SoftElementCode),
    // WriteMultipleWords(Address, Quantity, SoftElementCode),
    WriteMultipleBits(),
    WriteMultipleWords(),
}

impl Response {
    #[must_use]
    pub const fn function_code(&self) -> FunctionCode {
        use Response::*;

        match self {
            ReadBits(_) => FunctionCode::ReadBits,
            ReadWords(_) => FunctionCode::ReadWords,
            // WriteMultipleBits(_, _, _) => FunctionCode::WriteMultipleBits,
            // WriteMultipleWords(_, _, _) => FunctionCode::WriteMultipleWords,
            WriteMultipleBits() => FunctionCode::WriteMultipleBits,
            WriteMultipleWords() => FunctionCode::WriteMultipleWords,
        }
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
