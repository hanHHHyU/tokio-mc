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