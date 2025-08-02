// 优化的地址解析，针对Keyence地址格式
#[inline]
pub fn split_address(address: &str) -> Option<(&str, &str)> {
    // 快速路径：检查地址是否足够长
    if address.len() < 2 {
        return None;
    }

    let bytes = address.as_bytes();

    // 优化：处理双字符前缀的特殊情况
    let prefix_len = match (bytes.get(0), bytes.get(1), bytes.get(2)) {
        // 双字符前缀检查
        (Some(&b'D'), Some(&b'M'), Some(third))
            if third.is_ascii_digit() || third.is_ascii_alphanumeric() =>
        {
            2
        }
        (Some(&b'F'), Some(&b'M'), Some(third))
            if third.is_ascii_digit() || third.is_ascii_alphanumeric() =>
        {
            2
        }
        (Some(&b'M'), Some(&b'R'), Some(third))
            if third.is_ascii_digit() || third.is_ascii_alphanumeric() =>
        {
            2
        }
        (Some(&b'L'), Some(&b'R'), Some(third))
            if third.is_ascii_digit() || third.is_ascii_alphanumeric() =>
        {
            2
        }
        (Some(&b'C'), Some(&b'R'), Some(third))
            if third.is_ascii_digit() || third.is_ascii_alphanumeric() =>
        {
            2
        }
        (Some(&b'C'), Some(&b'M'), Some(third))
            if third.is_ascii_digit() || third.is_ascii_alphanumeric() =>
        {
            2
        }
        (Some(&b'E'), Some(&b'M'), Some(third))
            if third.is_ascii_digit() || third.is_ascii_alphanumeric() =>
        {
            2
        }
        (Some(&b'Z'), Some(&b'F'), Some(third))
            if third.is_ascii_digit() || third.is_ascii_alphanumeric() =>
        {
            2
        }
        // 单字符前缀检查
        (Some(&b'R'), Some(second), _)
            if second.is_ascii_digit() || second.is_ascii_alphanumeric() =>
        {
            1
        }
        (Some(&b'X'), Some(second), _)
            if second.is_ascii_digit() || second.is_ascii_alphanumeric() =>
        {
            1
        }
        (Some(&b'Y'), Some(second), _)
            if second.is_ascii_digit() || second.is_ascii_alphanumeric() =>
        {
            1
        }
        (Some(&b'B'), Some(second), _)
            if second.is_ascii_digit() || second.is_ascii_alphanumeric() =>
        {
            1
        }
        (Some(&b'T'), Some(second), _)
            if second.is_ascii_digit() || second.is_ascii_alphanumeric() =>
        {
            1
        }
        (Some(&b'C'), Some(second), _)
            if second.is_ascii_digit() || second.is_ascii_alphanumeric() =>
        {
            1
        }
        (Some(&b'M'), Some(second), _)
            if second.is_ascii_digit() || second.is_ascii_alphanumeric() =>
        {
            1
        }
        (Some(&b'L'), Some(second), _)
            if second.is_ascii_digit() || second.is_ascii_alphanumeric() =>
        {
            1
        }
        (Some(&b'D'), Some(second), _)
            if second.is_ascii_digit() || second.is_ascii_alphanumeric() =>
        {
            1
        }
        (Some(&b'F'), Some(second), _)
            if second.is_ascii_digit() || second.is_ascii_alphanumeric() =>
        {
            1
        }
        _ => 0,
    };

    if prefix_len > 0 {
        let (prefix, number) = address.split_at(prefix_len);

        // 验证数字部分是否有效
        if !number.is_empty()
            && number
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
        {
            Some((prefix, number))
        } else {
            None
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_keyence_address() {
        // 测试单字符前缀
        assert_eq!(split_address("R100"), Some(("R", "100")));
        assert_eq!(split_address("D200"), Some(("D", "200")));
        assert_eq!(split_address("M0"), Some(("M", "0")));

        // 测试双字符前缀
        assert_eq!(split_address("DM100"), Some(("DM", "100")));
        assert_eq!(split_address("FM200"), Some(("FM", "200")));
        assert_eq!(split_address("MR300"), Some(("MR", "300")));
        assert_eq!(split_address("LR50"), Some(("LR", "50")));
        assert_eq!(split_address("CR60"), Some(("CR", "60")));
        assert_eq!(split_address("CM70"), Some(("CM", "70")));
        assert_eq!(split_address("EM80"), Some(("EM", "80")));
        assert_eq!(split_address("ZF90"), Some(("ZF", "90")));

        // 测试带小数点的地址
        assert_eq!(split_address("R100.01"), Some(("R", "100.01")));
        assert_eq!(split_address("DM200.15"), Some(("DM", "200.15")));

        // 测试无效地址
        assert_eq!(split_address(""), None);
        assert_eq!(split_address("D"), None);
        assert_eq!(split_address("AB100"), None);
    }
}
