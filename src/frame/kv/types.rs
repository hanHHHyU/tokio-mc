#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataOProcess {
    // 不处理
    None,
    // 16进制处理 如R100 转换为 X10
    Hex,
    // 10进制处理
    Decimal,

    // 10进制转16进制处理
    DecimalToHex,

    XYToHex,
}