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
    ReadI16s,
    ReadU32s,
    ReadI32s,
    ReadF32s,
    ReadF64s,
    ReadU64s,
    ReadI64s,
    WriteBools,
    WriteU16s,
    WriteI16s,
    WriteU32s,
    WriteI32s,
    WriteF32s,
    WriteU64s,
    WriteI64s,
    WriteF64s,
}

impl FunctionCode {
    /// Create a new [`FunctionCode`] with `value`.
    #[must_use]
    pub fn new(value: BytesMut) -> Option<Self> {
        match &value[..] {
            [0x01, 0x04, 0x00, 0x00] => Some(Self::ReadBools), // 假设这对应 ReadBits
            // [0x01, 0x04, 0x00, 0x00] => Some(Self::ReadU16s),  // 假设这对应 ReadWords
            // [0x01, 0x04, 0x00, 0x00] => Some(Self::ReadI16s),
            [0x01, 0x14, 0x01, 0x00] => Some(Self::WriteBools), // 对应 WriteMultipleBits
            [0x01, 0x14, 0x00, 0x00] => Some(Self::WriteU16s),  // 对应 WriteMultipleWords
            // [0x01, 0x14, 0x00, 0x00] => Some(Self::WriteI16s),  // 对应 WriteMultipleWords
            _ => None, // 如果字节序列不匹配，返回 None
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
                buf.extend_from_slice(&[0x01, 0x04, 0x00, 0x00]);
            }
            FunctionCode::ReadI16s => {
                buf.extend_from_slice(&[0x01, 0x04, 0x00, 0x00]);
            }
            FunctionCode::ReadU32s => {
                buf.extend_from_slice(&[0x01, 0x04, 0x00, 0x00]);
            }
            FunctionCode::ReadI32s => {
                buf.extend_from_slice(&[0x01, 0x04, 0x00, 0x00]);
            }
            FunctionCode::ReadF32s => {
                buf.extend_from_slice(&[0x01, 0x04, 0x00, 0x00]);
            }
            FunctionCode::ReadF64s => {
                buf.extend_from_slice(&[0x01, 0x04, 0x00, 0x00]);
            }
            FunctionCode::ReadU64s => {
                buf.extend_from_slice(&[0x01, 0x04, 0x00, 0x00]);
            }
            FunctionCode::ReadI64s => {
                buf.extend_from_slice(&[0x01, 0x04, 0x00, 0x00]);
            }
            FunctionCode::WriteBools => {
                buf.extend_from_slice(&[0x01, 0x14, 0x01, 0x00]);
            }
            FunctionCode::WriteU16s => {
                buf.extend_from_slice(&[0x01, 0x14, 0x00, 0x00]);
            }
            FunctionCode::WriteI16s => {
                buf.extend_from_slice(&[0x01, 0x14, 0x00, 0x00]);
            }
            FunctionCode::WriteU32s => {
                buf.extend_from_slice(&[0x01, 0x14, 0x00, 0x00]);
            }
            FunctionCode::WriteI32s => {
                buf.extend_from_slice(&[0x01, 0x14, 0x00, 0x00]);
            }
            FunctionCode::WriteF32s => {
                buf.extend_from_slice(&[0x01, 0x14, 0x00, 0x00]);
            }
            FunctionCode::WriteU64s => {
                buf.extend_from_slice(&[0x01, 0x14, 0x00, 0x00]);
            }
            FunctionCode::WriteI64s => {
                buf.extend_from_slice(&[0x01, 0x14, 0x00, 0x00]);
            }
            FunctionCode::WriteF64s => {
                buf.extend_from_slice(&[0x01, 0x14, 0x00, 0x00]);
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
#[derive(Debug, Clone, PartialEq)]
pub enum Request<'a> {
    ReadBools(Cow<'a, str>, Quantity),
    ReadU16s(Cow<'a, str>, Quantity),
    ReadI16s(Cow<'a, str>, Quantity),
    ReadU32s(Cow<'a, str>, Quantity),
    ReadI32s(Cow<'a, str>, Quantity),
    ReadF32s(Cow<'a, str>, Quantity),
    ReadF64s(Cow<'a, str>, Quantity),
    ReadU64s(Cow<'a, str>, Quantity),
    ReadI64s(Cow<'a, str>, Quantity),
    WriteBools(Cow<'a, str>, Cow<'a, [bool]>),
    WriteU16s(Cow<'a, str>, Cow<'a, [u16]>),
    WriteI16s(Cow<'a, str>, Cow<'a, [i16]>),
    WriteU32s(Cow<'a, str>, Cow<'a, [u32]>),
    WriteI32s(Cow<'a, str>, Cow<'a, [i32]>),
    WriteF32s(Cow<'a, str>, Cow<'a, [f32]>),
    WriteU64s(Cow<'a, str>, Cow<'a, [u64]>),
    WriteI64s(Cow<'a, str>, Cow<'a, [i64]>),
    WriteF64s(Cow<'a, str>, Cow<'a, [f64]>),
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
            ReadI16s(addr, qty) => ReadI16s(Cow::Owned(addr.into_owned()), qty),
            ReadU32s(addr, qty) => ReadU32s(Cow::Owned(addr.into_owned()), qty),
            ReadI32s(addr, qty) => ReadI32s(Cow::Owned(addr.into_owned()), qty),
            ReadF32s(addr, qty) => ReadF32s(Cow::Owned(addr.into_owned()), qty),
            ReadF64s(addr, qty) => ReadF64s(Cow::Owned(addr.into_owned()), qty),
            ReadU64s(addr, qty) => ReadU64s(Cow::Owned(addr.into_owned()), qty),
            ReadI64s(addr, qty) => ReadU64s(Cow::Owned(addr.into_owned()), qty),
            WriteBools(addr, bools) => WriteBools(
                Cow::Owned(addr.into_owned()),
                Cow::Owned(bools.into_owned()),
            ),
            WriteU16s(addr, u16s) => {
                WriteU16s(Cow::Owned(addr.into_owned()), Cow::Owned(u16s.into_owned()))
            }
            WriteI16s(addr, i16s) => {
                WriteI16s(Cow::Owned(addr.into_owned()), Cow::Owned(i16s.into_owned()))
            }
            WriteU32s(addr, u32s) => {
                WriteU32s(Cow::Owned(addr.into_owned()), Cow::Owned(u32s.into_owned()))
            }
            WriteI32s(addr, i32s) => {
                WriteI32s(Cow::Owned(addr.into_owned()), Cow::Owned(i32s.into_owned()))
            }
            WriteF32s(addr, f32s) => {
                WriteF32s(Cow::Owned(addr.into_owned()), Cow::Owned(f32s.into_owned()))
            }

            WriteU64s(addr, u32s) => {
                WriteU64s(Cow::Owned(addr.into_owned()), Cow::Owned(u32s.into_owned()))
            }
            WriteI64s(addr, i32s) => {
                WriteI64s(Cow::Owned(addr.into_owned()), Cow::Owned(i32s.into_owned()))
            }
            WriteF64s(addr, f32s) => {
                WriteF64s(Cow::Owned(addr.into_owned()), Cow::Owned(f32s.into_owned()))
            }
        }
    }

    #[must_use]
    pub const fn function_code(&self) -> FunctionCode {
        use Request::*;
        match self {
            ReadBools(_, _) => FunctionCode::ReadBools,
            ReadU16s(_, _) => FunctionCode::ReadU16s,
            ReadI16s(_, _) => FunctionCode::ReadI16s,
            ReadU32s(_, _) => FunctionCode::ReadU32s,
            ReadI32s(_, _) => FunctionCode::ReadI32s,
            ReadF32s(_, _) => FunctionCode::ReadF32s,
            ReadF64s(_, _) => FunctionCode::ReadF64s,
            ReadU64s(_, _) => FunctionCode::ReadU64s,
            ReadI64s(_, _) => FunctionCode::ReadI64s,
            WriteBools(_, _) => FunctionCode::WriteBools,
            WriteU16s(_, _) => FunctionCode::WriteU16s,
            WriteI16s(_, _) => FunctionCode::WriteI16s,
            WriteU32s(_, _) => FunctionCode::WriteU32s,
            WriteI32s(_, _) => FunctionCode::WriteI32s,
            WriteF32s(_, _) => FunctionCode::WriteF32s,
            WriteU64s(_, _) => FunctionCode::WriteU64s,
            WriteI64s(_, _) => FunctionCode::WriteI64s,
            WriteF64s(_, _) => FunctionCode::WriteF64s,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Response {
    ReadBools(Vec<bool>),
    ReadU16s(Vec<u16>),
    ReadI16s(Vec<i16>),
    ReadU32s(Vec<u32>),
    ReadI32s(Vec<i32>),
    ReadF32s(Vec<f32>),
    ReadF64s(Vec<f64>),
    ReadU64s(Vec<u64>),
    ReadI64s(Vec<i64>),
    // WriteMultipleBits(Address, Quantity, SoftElementCode),
    // WriteMultipleWords(Address, Quantity, SoftElementCode),
    WriteBools(),
    WriteU16s(),
    WriteI16s(),
    WriteU32s(),
    WriteI32s(),
    WriteF32s(),
    WriteU64s(),
    WriteI64s(),
    WriteF64s(),
}

pub struct ResponseIterator {
    response: Response,
}

impl ResponseIterator {
    pub fn new(response: Response) -> Self {
        ResponseIterator { response }
    }
}

impl Iterator for ResponseIterator {
    type Item = Box<dyn std::fmt::Debug>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.response {
            Response::ReadI16s(data) => data
                .pop()
                .map(|val| Box::new(val) as Box<dyn std::fmt::Debug>),
            Response::ReadU16s(data) => data
                .pop()
                .map(|val| Box::new(val) as Box<dyn std::fmt::Debug>),
            Response::ReadBools(data) => data
                .pop()
                .map(|val| Box::new(val) as Box<dyn std::fmt::Debug>),
            Response::ReadU32s(data) => data
                .pop()
                .map(|val| Box::new(val) as Box<dyn std::fmt::Debug>),
            Response::ReadI32s(data) => data
                .pop()
                .map(|val| Box::new(val) as Box<dyn std::fmt::Debug>),
            Response::ReadF32s(data) => data
                .pop()
                .map(|val| Box::new(val) as Box<dyn std::fmt::Debug>),
            Response::ReadF64s(data) => data
                .pop()
                .map(|val| Box::new(val) as Box<dyn std::fmt::Debug>),
            Response::ReadU64s(data) => data
                .pop()
                .map(|val| Box::new(val) as Box<dyn std::fmt::Debug>),
            Response::ReadI64s(data) => data
                .pop()
                .map(|val| Box::new(val) as Box<dyn std::fmt::Debug>),
            _ => None,
        }
    }
}

impl Response {
    #[must_use]
    pub const fn function_code(&self) -> FunctionCode {
        use Response::*;

        match self {
            ReadBools(_) => FunctionCode::ReadBools,
            ReadU16s(_) => FunctionCode::ReadU16s,
            ReadI16s(_) => FunctionCode::ReadI16s,
            ReadU32s(_) => FunctionCode::ReadU32s,
            ReadI32s(_) => FunctionCode::ReadI32s,
            ReadF32s(_) => FunctionCode::ReadF32s,
            ReadF64s(_) => FunctionCode::ReadF64s,
            ReadU64s(_) => FunctionCode::ReadU64s,
            ReadI64s(_) => FunctionCode::ReadI64s,
            // WriteMultipleBits(_, _, _) => FunctionCode::WriteMultipleBits,
            // WriteMultipleWords(_, _, _) => FunctionCode::WriteMultipleWords,
            WriteBools() => FunctionCode::WriteBools,
            WriteU16s() => FunctionCode::WriteU16s,
            WriteI16s() => FunctionCode::WriteI16s,
            WriteU32s() => FunctionCode::WriteU32s,
            WriteI32s() => FunctionCode::WriteI32s,
            WriteF32s() => FunctionCode::WriteF32s,
            WriteI64s() => FunctionCode::WriteI64s,
            WriteU64s() => FunctionCode::WriteU64s,
            WriteF64s() => FunctionCode::WriteF64s,
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
            FunctionCode::new(BytesMut::from(&[0x01, 0x04, 0x00, 0x00][..]))
                .expect("Failed to create FunctionCode from bytes")
        );
        assert_eq!(
            FunctionCode::ReadBools,
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
        let read_bits_bytes = BytesMut::from(&[0x01, 0x04, 0x00, 0x00][..]);
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
            FunctionCode::ReadBools.value(),
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
