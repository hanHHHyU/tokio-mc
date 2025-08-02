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

pub use map::{convert_to_base, find_instruction_code, find_prefix_and_base_by_code};
pub use regex::split_address;

pub use kv::convert_keyence_to_mitsubishi_address;

pub use kv::KVError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionCode {
    ReadU8s,
    WriteU8s,
    ReadBits,
    WriteBits,
}

impl FunctionCode {
    /// Create a new [`FunctionCode`] with `value`.
    #[must_use]
    pub fn new(value: BytesMut) -> Option<Self> {
        match &value[..] {
            // MC协议格式: [指令代码(2字节), 子指令代码(2字节)]
            [0x01, 0x04, 0x00, 0x00] => Some(Self::ReadU8s), // 兼容旧格式
            [0x01, 0x14, 0x00, 0x00] => Some(Self::WriteU8s), // 兼容旧格式
            [0x01, 0x04, 0x01, 0x00] => Some(Self::ReadBits), // bit读取
            [0x01, 0x14, 0x01, 0x00] => Some(Self::WriteBits), // bit写入
            _ => None,
        }
    }

    /// 将 `FunctionCode` 转换为相应的 `BytesMut` 字节序列
    #[must_use]
    pub fn value(self) -> BytesMut {
        let mut buf = BytesMut::new();
        match self {
            FunctionCode::ReadU8s => {
                buf.extend_from_slice(&[0x01, 0x04, 0x00, 0x00]);
            }
            FunctionCode::WriteU8s => {
                buf.extend_from_slice(&[0x01, 0x14, 0x00, 0x00]);
            }
            FunctionCode::ReadBits => {
                buf.extend_from_slice(&[0x01, 0x04, 0x01, 0x00]);
            }
            FunctionCode::WriteBits => {
                buf.extend_from_slice(&[0x01, 0x14, 0x01, 0x00]);
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
    ReadU8s(Cow<'a, str>, Quantity),
    WriteU8s(Cow<'a, str>, Cow<'a, [u8]>),
    ReadBits(Cow<'a, str>, Quantity),
    WriteBits(Cow<'a, str>, Cow<'a, [bool]>),
}

// 实现辅助功能，比如将请求转换为'owned'版本或获取功能码
impl<'a> Request<'a> {
    /// 将请求转换为'owned'的实例（静态生命周期）
    #[must_use]
    pub fn into_owned(self) -> Request<'static> {
        use Request::*;
        match self {
            ReadU8s(addr, qty) => ReadU8s(Cow::Owned(addr.into_owned()), qty),
            WriteU8s(addr, u8s) => {
                WriteU8s(Cow::Owned(addr.into_owned()), Cow::Owned(u8s.into_owned()))
            }
            ReadBits(addr, qty) => ReadBits(Cow::Owned(addr.into_owned()), qty),
            WriteBits(addr, bits) => {
                WriteBits(Cow::Owned(addr.into_owned()), Cow::Owned(bits.into_owned()))
            }
        }
    }

    #[must_use]
    pub const fn function_code(&self) -> FunctionCode {
        use Request::*;
        match self {
            ReadU8s(_, _) => FunctionCode::ReadU8s,
            WriteU8s(_, _) => FunctionCode::WriteU8s,
            ReadBits(_, _) => FunctionCode::ReadBits,
            WriteBits(_, _) => FunctionCode::WriteBits,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Response {
    ReadU8s(Vec<u8>),
    WriteU8s(),
    ReadBits(Vec<bool>),
    WriteBits(),
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
            Response::ReadU8s(data) => data
                .pop()
                .map(|val| Box::new(val) as Box<dyn std::fmt::Debug>),
            Response::ReadBits(data) => data
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
            ReadU8s(_) => FunctionCode::ReadU8s,
            WriteU8s() => FunctionCode::WriteU8s,
            ReadBits(_) => FunctionCode::ReadBits,
            WriteBits() => FunctionCode::WriteBits,
        }
    }

    // 获取长度
    pub fn len(&self) -> usize {
        match self {
            Response::ReadU8s(values) => values.len() / 2,
            Response::WriteU8s() => 0,
            Response::ReadBits(values) => values.len(),
            Response::WriteBits() => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_function_code() {
        // 测试旧格式兼容性
        assert_eq!(
            FunctionCode::ReadU8s,
            FunctionCode::new(BytesMut::from(&[0x01, 0x04, 0x00, 0x00][..]))
                .expect("Failed to create FunctionCode from legacy bytes")
        );
        assert_eq!(
            FunctionCode::WriteU8s,
            FunctionCode::new(BytesMut::from(&[0x01, 0x14, 0x00, 0x00][..]))
                .expect("Failed to create FunctionCode from legacy bytes")
        );

        // 测试bit操作
        assert_eq!(
            FunctionCode::ReadBits,
            FunctionCode::new(BytesMut::from(&[0x01, 0x04, 0x01, 0x00][..]))
                .expect("Failed to create FunctionCode for ReadBits")
        );
        assert_eq!(
            FunctionCode::WriteBits,
            FunctionCode::new(BytesMut::from(&[0x01, 0x14, 0x01, 0x00][..]))
                .expect("Failed to create FunctionCode for WriteBits")
        );
    }

    #[test]
    fn function_code_values() {
        let read_u8s_bytes = BytesMut::from(&[0x01, 0x04, 0x00, 0x00][..]);
        let write_u8s_bytes = BytesMut::from(&[0x01, 0x14, 0x00, 0x00][..]);
        let read_bits_bytes = BytesMut::from(&[0x01, 0x04, 0x01, 0x00][..]);
        let write_bits_bytes = BytesMut::from(&[0x01, 0x14, 0x01, 0x00][..]);

        assert_eq!(
            FunctionCode::ReadU8s.value(),
            read_u8s_bytes,
            "ReadU8s byte sequence is incorrect"
        );

        assert_eq!(
            FunctionCode::WriteU8s.value(),
            write_u8s_bytes,
            "WriteU8s byte sequence is incorrect"
        );

        assert_eq!(
            FunctionCode::ReadBits.value(),
            read_bits_bytes,
            "ReadBits byte sequence is incorrect"
        );

        assert_eq!(
            FunctionCode::WriteBits.value(),
            write_bits_bytes,
            "WriteBits byte sequence is incorrect"
        );
    }
}
