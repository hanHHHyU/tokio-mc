use super::NumberBase;

// 优化：使用静态数组代替HashMap，提高查找性能
const PLC_INSTRUCTIONS: &[(&str, u8, NumberBase)] = &[
    ("X", 0x9c, NumberBase::Hexadecimal),
    ("Y", 0x9d, NumberBase::Hexadecimal),
    ("F", 0x93, NumberBase::Decimal),
    ("M", 0x90, NumberBase::Decimal),
    ("L", 0x92, NumberBase::Decimal),
    ("D", 0xa8, NumberBase::Decimal),
    ("R", 0xaf, NumberBase::Decimal),
    ("B", 0xA0, NumberBase::Hexadecimal),
    ("SM", 0x91, NumberBase::Decimal),     // 特殊继电器
    ("SD", 0xA9, NumberBase::Decimal),     // 特殊存储器
    ("ZR", 0xB0, NumberBase::Hexadecimal), // 文件寄存器
    ("W", 0xB4, NumberBase::Hexadecimal),  // 链接寄存器
    ("TN", 0xC2, NumberBase::Decimal),     // 定时器当前值
    ("TS", 0xC1, NumberBase::Decimal),     // 定时器接点
    ("CN", 0xC5, NumberBase::Decimal),     // 计数器当前值
    ("CS", 0xC4, NumberBase::Decimal),     // 计数器接点
];

// 优化的查找函数，使用线性搜索（对于小数组更快）
#[inline]
pub fn find_instruction_code(prefix: &str) -> Option<(u8, NumberBase)> {
    PLC_INSTRUCTIONS
        .iter()
        .find(|(p, _, _)| *p == prefix)
        .map(|(_, code, base)| (*code, *base))
}

// 优化的数字转换，处理常见情况
#[inline]
pub fn convert_to_base(s: &str, number_base: NumberBase) -> Option<u32> {
    match number_base {
        NumberBase::Decimal => {
            // 快速路径：纯数字解析
            let mut result = 0u32;
            for byte in s.bytes() {
                match byte {
                    b'0'..=b'9' => {
                        result = result.checked_mul(10)?;
                        result = result.checked_add((byte - b'0') as u32)?;
                    }
                    _ => return None,
                }
            }
            Some(result)
        }
        NumberBase::Hexadecimal => {
            // 快速路径：十六进制解析
            let mut result = 0u32;
            for byte in s.bytes() {
                let digit = match byte {
                    b'0'..=b'9' => (byte - b'0') as u32,
                    b'A'..=b'F' => (byte - b'A' + 10) as u32,
                    b'a'..=b'f' => (byte - b'a' + 10) as u32,
                    _ => return None,
                };
                result = result.checked_mul(16)?;
                result = result.checked_add(digit)?;
            }
            Some(result)
        }
    }
}

// 优化的反向查找
#[inline]
pub fn find_prefix_and_base_by_code(code: u8) -> Option<(&'static str, NumberBase)> {
    PLC_INSTRUCTIONS
        .iter()
        .find(|(_, c, _)| *c == code)
        .map(|(prefix, _, base)| (*prefix, *base))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_instruction_code() {
        assert_eq!(
            find_instruction_code("D"),
            Some((0xa8, NumberBase::Decimal))
        );
        assert_eq!(
            find_instruction_code("X"),
            Some((0x9c, NumberBase::Hexadecimal))
        );
        assert_eq!(
            find_instruction_code("SM"),
            Some((0x91, NumberBase::Decimal))
        );
        assert_eq!(
            find_instruction_code("ZR"),
            Some((0xB0, NumberBase::Hexadecimal))
        );
        assert_eq!(find_instruction_code("INVALID"), None);
    }

    #[test]
    fn test_convert_to_base() {
        // 十进制测试
        assert_eq!(convert_to_base("100", NumberBase::Decimal), Some(100));
        assert_eq!(convert_to_base("0", NumberBase::Decimal), Some(0));
        assert_eq!(
            convert_to_base("4294967295", NumberBase::Decimal),
            Some(4294967295)
        );
        assert_eq!(convert_to_base("4294967296", NumberBase::Decimal), None); // 溢出
        assert_eq!(convert_to_base("abc", NumberBase::Decimal), None);

        // 十六进制测试
        assert_eq!(convert_to_base("FF", NumberBase::Hexadecimal), Some(255));
        assert_eq!(convert_to_base("ff", NumberBase::Hexadecimal), Some(255));
        assert_eq!(convert_to_base("A0", NumberBase::Hexadecimal), Some(160));
        assert_eq!(
            convert_to_base("FFFFFFFF", NumberBase::Hexadecimal),
            Some(4294967295)
        );
        assert_eq!(convert_to_base("100000000", NumberBase::Hexadecimal), None); // 溢出
        assert_eq!(convert_to_base("XYZ", NumberBase::Hexadecimal), None);
    }

    #[test]
    fn test_find_prefix_and_base_by_code() {
        assert_eq!(
            find_prefix_and_base_by_code(0xa8),
            Some(("D", NumberBase::Decimal))
        );
        assert_eq!(
            find_prefix_and_base_by_code(0x9c),
            Some(("X", NumberBase::Hexadecimal))
        );
        assert_eq!(find_prefix_and_base_by_code(0xFF), None);
    }
}
