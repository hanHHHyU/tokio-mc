use convert::convert_xy_number;
pub use error::KVError;
use regex::split_address;
use map::find;
use types::DataOProcess;

mod convert;
mod map;
mod regex;
mod types;
mod error;



pub fn convert_keyence_to_mitsubishi_address(address: &str) -> Result<String, KVError> {
    let (prefix, address) = split_address(address).ok_or(KVError::PaseError)?;
    let (instruction, process) = find(prefix).ok_or(KVError::MapNotFound)?;

    match process {
        DataOProcess::Hex | DataOProcess::Decimal => {
            let address = address
                .parse::<u32>()
                .map_err(|_| KVError::ParseNumberError)?;
            let (resul1, result2) = (address % 100, (address - address % 100) / 100);

            if resul1 > 16 {
                return Err(KVError::AddressInvalid);
            }

            let formatted_result1 = format!("{:X}", resul1);
            let formatted_result2 = if result2 == 0 && process == DataOProcess::Hex {
                "".to_string()
            } else {
                format!("{:X}", result2)
            };

            Ok(if process == DataOProcess::Hex {
                format!("{}{}{}", instruction, formatted_result2, formatted_result1)
            } else {
                // Convert Hex to Decimal
                let decimal =
                    u32::from_str_radix(&format!("{}{}", formatted_result2, formatted_result1), 16)
                        .map_err(|_| KVError::ConvertError)?;

                format!("{}{}", instruction, decimal)
            })
        }
        DataOProcess::DecimalToHex => {
            let address = address
                .parse::<u32>()
                .map_err(|_| KVError::ParseNumberError)?;
            // 将address转换为16进制
            let formatted_address = format!("{:X}", address);
            Ok(instruction.to_owned() + &formatted_address)
        }
        DataOProcess::XYToHex => Ok(instruction.to_owned() + &convert_xy_number(address)?),

        DataOProcess::None => Ok(instruction.to_owned() + address),
    }
}



#[cfg(test)]
mod tests {
    use super::*; // 引入当前模块的所有项，假设 `convert_keyence_to_mitsubishi_address` 在当前模块内

    // 测试 convert_keyence_to_mitsubishi_address 函数
    #[test]
    fn test_convert_keyence_to_mitsubishi_address_hex() {
        let address = "X101";
        let result = convert_keyence_to_mitsubishi_address(address);

        // 打印 result 以查看实际输出
        println!("Result: {:?}", result);

        // 你可以根据需要添加断言
        assert!(result.is_ok());  // 只是一个示例，实际断言内容要根据函数的预期行为来定
    }

}
