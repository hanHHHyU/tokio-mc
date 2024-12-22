use super::error::KVError;

/// 将数字转换为指定规则的16进制格式
pub fn convert_xy_number(number: &str) -> Result<String, KVError> {
    // // 如果hexString只有一位，则直接返回number
    // if number.len() == 1 {
    //     let result = i32::from_str_radix(number, 16).unwrap();
    //     return format!("{:X}", result);
    // }
    // 如果 `number` 只有一位，直接返回其 16 进制表示
    if number.len() == 1 {
        return i32::from_str_radix(number, 16)
            .map(|n| format!("{:X}", n)) // 转换为 16 进制字符串
            .map_err(|e| KVError::InvalidNumberFormat {
                input: number.to_string(),
                source: e,
            });
    }

    // 分离最后一个字符
    let last_char = &number[number.len() - 1..];
    let remaining_chars = &number[..number.len() - 1];

    // 将剩余字符转换为整数并除以10
    let p: i32 = remaining_chars.parse::<i32>().unwrap_or(0);

    // 将结果转换回16进制字符串，并加上最后一个字符
    let hex_value = format!("{:X}", p);
    let result = format!("{}{}", hex_value, last_char);

    // 解析结果为整数
    let final_result =
        i32::from_str_radix(&result, 16).map_err(|e| KVError::InvalidNumberFormat {
            input: result.clone(),
            source: e,
        })?;

    // 转换最终整数为 16 进制字符串并返回
    Ok(format!("{:X}", final_result))
}

#[cfg(test)]
mod tests {
    use super::*; // 引入当前模块的所有项

    #[test]
    fn test_convert_xy_number() {
        let input = "100";
        let result = convert_xy_number(input);
        assert_eq!(result.unwrap(), "A0");

        let input = "00F";
        let result = convert_xy_number(input);
        assert_eq!(result.unwrap(), "F");

        let input = "20F";
        let result = convert_xy_number(input);
        assert_eq!(result.unwrap(), "14F");

        let input = "300";
        let result = convert_xy_number(input);
        assert_eq!(result.unwrap(), "1E0");

        let input = "1000";
        let result = convert_xy_number(input);
        assert_eq!(result.unwrap(), "640");

        let input = "100A";
        let result = convert_xy_number(input);
        assert_eq!(result.unwrap(), "64A");

        // 添加无效输入测试
        let input = "XYZ";
        let result = convert_xy_number(input);
        assert!(result.is_err(), "Expected an error for invalid input");
    }
}
