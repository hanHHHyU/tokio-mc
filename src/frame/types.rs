use std::{borrow::Cow, io::Cursor};
// use tokio::time::Duration;

pub type Quantity = u32;

pub(crate) const REQUEST_BYTE_LAST_LEN: usize = 10;

pub(crate) const LIMIT: u32 = 900;

// // 定义一个全局常量来表示超时时间
// pub(crate) const TIMEOUT_DURATION: Duration = Duration::from_millis(100);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NumberBase {
    /// The decimal numbering system base (base 10).
    ///
    /// This variant represents numbers using the standard 0-9 digits.
    Decimal,

    /// The hexadecimal numbering system base (base 16).
    ///
    /// This variant represents numbers using 0-9 digits and A-F letters.
    Hexadecimal,
}

pub struct PlcInstruction {
    pub(crate) code: u8,
    pub(crate) number_base: NumberBase,
}

pub enum WriteCursor<'a> {
    Bools(Cursor<Cow<'a, [u8]>>),
    U8s(Cursor<Cow<'a, [u8]>>),
}

#[derive(Debug, Clone, Copy)]
pub enum Model {
    Mitsubishi,
    Keyence,
}

impl Default for Model {
    fn default() -> Self {
        Model::Mitsubishi
    }
}
