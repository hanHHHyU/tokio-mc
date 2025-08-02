pub type Quantity = u32;

pub(crate) const REQUEST_BYTE_LAST_LEN: usize = 10;

pub(crate) const LIMIT: u32 = 960;

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
