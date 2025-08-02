// 优化的地址解析，避免使用正则表达式
#[inline]
pub fn split_address(address: &str) -> Option<(&str, &str)> {
    // 快速路径：检查地址是否足够长
    if address.len() < 2 {
        return None;
    }

    let bytes = address.as_bytes();

    // 优化：处理双字符前缀的特殊情况
    let prefix_len = match (bytes.get(0), bytes.get(1), bytes.get(2)) {
        // 双字符前缀检查（必须先检查，否则会被单字符匹配）
        (Some(&b'S'), Some(&b'M'), Some(third))
            if third.is_ascii_digit() || third.is_ascii_alphanumeric() =>
        {
            2
        }
        (Some(&b'S'), Some(&b'D'), Some(third))
            if third.is_ascii_digit() || third.is_ascii_alphanumeric() =>
        {
            2
        }
        (Some(&b'Z'), Some(&b'R'), Some(third))
            if third.is_ascii_digit() || third.is_ascii_alphanumeric() =>
        {
            2
        }
        (Some(&b'T'), Some(&b'N'), Some(third))
            if third.is_ascii_digit() || third.is_ascii_alphanumeric() =>
        {
            2
        }
        (Some(&b'T'), Some(&b'S'), Some(third))
            if third.is_ascii_digit() || third.is_ascii_alphanumeric() =>
        {
            2
        }
        (Some(&b'C'), Some(&b'N'), Some(third))
            if third.is_ascii_digit() || third.is_ascii_alphanumeric() =>
        {
            2
        }
        (Some(&b'C'), Some(&b'S'), Some(third))
            if third.is_ascii_digit() || third.is_ascii_alphanumeric() =>
        {
            2
        }
        // 单字符前缀检查
        (Some(&b'X'), Some(second), _) if second.is_ascii_alphanumeric() => 1,
        (Some(&b'Y'), Some(second), _) if second.is_ascii_alphanumeric() => 1,
        (Some(&b'F'), Some(second), _) if second.is_ascii_digit() => 1,
        (Some(&b'B'), Some(second), _) if second.is_ascii_alphanumeric() => 1,
        (Some(&b'M'), Some(second), _) if second.is_ascii_digit() => 1,
        (Some(&b'L'), Some(second), _) if second.is_ascii_digit() => 1,
        (Some(&b'D'), Some(second), _) if second.is_ascii_digit() || second == &b'-' => 1,
        (Some(&b'R'), Some(second), _) if second.is_ascii_digit() => 1,
        (Some(&b'W'), Some(second), _) if second.is_ascii_alphanumeric() => 1,
        _ => 0,
    };

    if prefix_len > 0 {
        let (prefix, number) = address.split_at(prefix_len);

        // 验证数字部分是否有效
        if !number.is_empty() {
            Some((prefix, number))
        } else {
            None
        }
    } else {
        None
    }
}
