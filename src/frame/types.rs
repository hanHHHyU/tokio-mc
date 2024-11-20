use std::{borrow::Cow, io::Cursor};

pub(crate) type Bit = bool;

pub(crate) type Word = u16;

pub type Quantity = u16;

pub(crate) const REQUEST_BYTE_LAST_LEN: usize = 10;

pub(crate) const LIMIT: u16 = 900;

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
    Bits(Cursor<Cow<'a, [Bit]>>),
    Words(Cursor<Cow<'a, [Word]>>),
}
