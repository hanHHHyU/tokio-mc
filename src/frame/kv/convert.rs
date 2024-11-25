use super::error::KVError;

pub fn convert_xy_number(number: &str) -> Result<String, KVError> {
    // 如果 `number` 只有一位，或后续全部为 0，则直接返回解析的十进制字符串
    if number.len() == 1 || number[1..].chars().all(|c| c == '0') {
        return i32::from_str_radix(number, 16)
            .map(|n| n.to_string()) // 转换为字符串
            .map_err(|e| KVError::InvalidNumberFormat {
                input: number.to_string(),
                source: e,
            }); // 添加上下文并转换错误
    }

    // 获取 `number` 的最后一个字符和剩余部分
    let last_char = &number[number.len() - 1..];
    let remaining_chars = &number[..number.len() - 1];

    // 尝试解析剩余部分为整数
    let p: i32 = remaining_chars
        .parse() // 转换为 i32
        .map_err(|e| KVError::InvalidNumberFormat {
            input: remaining_chars.to_string(),
            source: e,
        })?; // 捕获并转换解析错误

    // 将结果转换回 16 进制字符串，并加上最后一个字符
    let hex_value = format!("{:X}", p);
    Ok(hex_value + last_char)
}
