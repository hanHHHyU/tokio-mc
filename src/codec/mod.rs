use std::{convert::TryFrom, io::Cursor};

use byteorder::{ByteOrder, LittleEndian, ReadBytesExt as _};

use crate::{
    bytes::{BufMut, Bytes, BytesMut},
    frame::*,
    header::RequestHeader,
    Error,
};

impl<'a> TryFrom<Request<'a>> for Vec<Bytes> {
    type Error = Error;

    #[allow(clippy::panic_in_result_fn)] // Intentional unreachable!()
    fn try_from(req: Request<'a>) -> Result<Vec<Bytes>, Error> {
        use crate::frame::Request::*;
        let header = RequestHeader::new();
        let cnt: usize = request_byte_count(&req, header.len());

        // 获取通用的地址、代码和进制数
        let (address, quantity_or_len, write_cursor) = match req {
            ReadBools(ref address, quantity) => {
                let adjusted_quantity = (quantity as f64 / 16.0).ceil() as u32;
                (address.clone(), adjusted_quantity, None)
            }
            ReadU16s(ref address, quantity) => (address.clone(), quantity, None),
            WriteBools(ref address, ref bits) => {
                let cursor = Cursor::new(bits.clone()); // 转换为 Cursor::new
                (
                    address.clone(),
                    bits.len().try_into().unwrap(),
                    Some(WriteCursor::Bits(cursor)),
                )
            }
            WriteU16s(ref address, ref words) => {
                let cursor = Cursor::new(words.clone()); // 转换为 Cursor::new
                (
                    address.clone(),
                    words.len().try_into().unwrap(),
                    Some(WriteCursor::Words(cursor)),
                )
            }
        };

        // // 获取地址、数量和写入数据
        // let (address, quantity_or_len, write_iter) = prepare_request_data(&req)?;

        let mut results = Vec::new();

        let (u32_number, code) = parse_address_and_get_instruction_code(&address)?;

        let mut current_len = quantity_or_len;

        let mut current_address = u32_number;

        // 如果有写入操作的 Cursor，则处理它
        let mut write_iter = match write_cursor {
            Some(WriteCursor::Bits(ref cursor)) => {
                // 使用 bools_to_bytes 转换布尔数组为字节数组
                let bytes = bools_to_bytes(cursor.get_ref());
                bytes.into_iter()
            }
            Some(WriteCursor::Words(ref cursor)) => {
                let mut bytes = Vec::with_capacity((cursor.get_ref().len() * 2) as usize);
                // 遍历 cursor 的内容，将每个 u16 转换为小端字节序
                for &word in cursor.get_ref().iter() {
                    bytes.extend_from_slice(&word.to_le_bytes());
                }
                bytes.into_iter()
            }
            None => vec![].into_iter(),
        };

        let header = RequestHeader::new();

        while current_len > 0 {
            let len = current_len.min(LIMIT) as u16;
            let mut data = BytesMut::with_capacity(cnt);
            data.put_slice(header.bytes());
            data.put_slice(&req.function_code().value());
            request_command(&mut data, current_address, code, len);

            // println!("读取处理长度 {:?}", len * 2);

            // 写入数据部分
            if let Some(write_cursor) = &write_cursor {
                match write_cursor {
                    WriteCursor::Bits(_) => {
                        println!("{}", ((len as f64) / 2.0).ceil() as u16);

                        for _ in 0..((len as f64 / 2.0).ceil() as u16) {
                            if let Some(value) = write_iter.next() {
                                data.put_u8(value); // 将每个字节放入数据块
                            }
                        }
                    }
                    WriteCursor::Words(_) => {
                        // Words 类型，长度处理为 len / 2
                        for _ in 0..len * 2 {
                            if let Some(value) = write_iter.next() {
                                data.put_u8(value); // 将每个字节放入数据块
                            }
                        }
                    }
                }
            }

            let length = (data.len() - header.len() + 2) as u16;

            // 使用 LittleEndian 的 `write_u16` 方法
            LittleEndian::write_u16(&mut data[header.len() - 4..header.len() - 2], length);

            current_address += len as u32;
            current_len = current_len.saturating_sub(len as u32);
            results.push(data.freeze());
        }

        Ok(results)
    }
}

impl TryFrom<(Vec<Bytes>, Request<'_>)> for Response {
    type Error = Error;
    fn try_from((bytes, req): (Vec<Bytes>, Request)) -> Result<Self, Error> {
        // let header = ResponseHeader::new();
        let mut data = Vec::new();

        for byte in &bytes {
            // // 检查响应数据的有效性
            // // println!("{:?}", byte);
            check_response(&byte)?;

            // println!("headerLen{}", header.len());

            // 跳过帧头部分并将数据追加到 data 中
            // data.extend_from_slice(&byte[header.len()..]);
            data.extend_from_slice(&byte[2..]);
        }
        // 处理 data 中的累积数据
        let mut final_rdr = Cursor::new(data);

        // 根据请求类型解析响应数据
        match req {
            Request::ReadBools(_, quantity) => {
                let total_bits = quantity as usize;
                let data = final_rdr.get_ref();

                // 使用迭代器生成 bits 向量
                let bits: Vec<bool> = (0..total_bits)
                    .map(|i| {
                        let byte_index = i / 8;
                        let bit_index = i % 8;
                        // 提取当前位的布尔值
                        (data[byte_index] >> bit_index) & 1 == 1
                    })
                    .collect();

                Ok(Response::ReadBools(bits))
            }
            Request::ReadU16s(_, quantity) => {
                let mut words = Vec::with_capacity(quantity as usize);
                for _ in 0..quantity {
                    // 读取小端字节序的 u16 值并放入 words 向量g
                    let word = final_rdr.read_u16::<LittleEndian>()?;
                    words.push(word);
                }

                Ok(Response::ReadU16s(words))
            }
            Request::WriteBools(_, _) => Ok(Response::WriteBools()),
            Request::WriteU16s(_, _) => Ok(Response::WriteU16s()),
        }
    }
}

fn request_byte_count(req: &Request<'_>, header_len: usize) -> usize {
    use crate::frame::Request::*;
    match *req {
        ReadBools(_, _) | ReadU16s(_, _) => header_len + REQUEST_BYTE_LAST_LEN,
        WriteBools(_, ref bits) => header_len + REQUEST_BYTE_LAST_LEN + (bits.len() + 1) / 2,
        WriteU16s(_, ref words) => header_len + REQUEST_BYTE_LAST_LEN + words.len() * 2,
    }
}

fn request_command(data: &mut BytesMut, address: u32, code: u8, cnt: u16) {
    assert!(address <= 0xFFFFFF, "Address out of range for u24");
    data.put_u16_le((address & 0xFFFF) as u16);
    data.put_u8((address >> 16) as u8); // 高位字节
    data.put_u8(code);
    data.put_u16_le(cnt);
}

fn parse_address_and_get_instruction_code(address: &str) -> Result<(u32, u8), Error> {
    let (prefix, number) = split_address(address).unwrap();

    let (code, number_base) = find_instruction_code(prefix).unwrap();

    let u32_number = convert_to_base(number, number_base).unwrap();

    Ok((u32_number, code))
}

fn check_response(response_bytes: &[u8]) -> Result<(), Error> {
    // let header_len = ResponseHeader::new().len();
    // 获取响应字节缓冲区的前 `header_len` 字节，并提取最后两个字节
    let last_two_bytes = &response_bytes[..2];
    // 将最后两个字节转换为小端格式的 16 位整数
    let last_two = LittleEndian::read_u16(last_two_bytes);

    if let Some(error) = map_error_code(last_two) {
        return Err(error.into());
    }

    Ok(())
}

pub fn bools_to_bytes(bools: &[bool]) -> Vec<u8> {
    bools
        .chunks(2)
        .map(|chunk| {
            // 判断布尔数组的长度，对每个chunk进行处理
            if chunk.len() == 2 {
                // 如果chunk长度为2，正常处理
                (chunk[0] as u8) << 4 | (chunk[1] as u8)
            } else {
                // 如果chunk长度为1（即数组长度为奇数，最后剩余一个布尔值）
                (chunk[0] as u8) << 4
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn test_read_bits_to_bytes() {
        // 构造一个 ReadBits 请求
        let request = Request::ReadBools("X0".to_owned().into(), 10);

        // 调用 try_from，尝试将 Request 转换为 Bytes
        let result = Vec::try_from(request);

        // 验证转换成功
        assert!(result.is_ok());

        // 获取转换后的字节数据
        let bytes = result.unwrap();

        // 检查返回的字节向量是否符合预期
        assert_eq!(bytes.len(), 1); // 假设一次循环能处理 32 个字节

        // 预期的字节数据，手动计算的结果
        let expected_bytes = vec![
            // 0x50, 0x00, // 3E 00 为MC协议的固定头
            0x00, // 00(网路编号) ：上位访问下位，固定00；
            0xFF, // FF(PLC编号) ：上位访问下位，固定FF；
            0xFF, 0x03, // 03(目标模块IO编号) ：上位访问下位，固定03；
            0x00, // 00(目标模块站号) ：上位访问下位，固定00；
            0x0C, 0x00, // 0x0C 为请求数据的长度
            0x10, 0x00, //
            0x01, 0x04, // 01 04 为读取命令
            0x00, 0x00, // 按字读取，如果按位读取则为 0x01 0x00
            0x00, 0x00, 0x00, // 起始地址 50
            0x9C, // 软元件代码9C为X为软元件代码
            0x01, 0x00, // 读取的软元件点数
        ];

        // 验证第一个字节数组是否与预期匹配bytes
        assert_eq!(
            bytes[0].to_vec(),
            expected_bytes,
            "The first byte block does not match the expected bytes"
        );

        // 打印调试信息
        println!("Generated bytes: {:?}", bytes[0].to_vec());
        println!("Expected bytes: {:?}", expected_bytes);
    }

    #[test]
    fn test_read_words_to_bytes() {
        // 构造一个 ReadWords 请求
        let request = Request::ReadU16s("D0".to_owned().into(), 901);

        // 调用 try_from，尝试将 Request 转换为 Bytes
        let result = Vec::try_from(request.clone()).unwrap();
        // 验证结果是否符合预期
        assert!(!result.is_empty(), "Result should not be empty");

        // 检查返回的字节向量是否符合预期
        assert_eq!(result.len(), 2); // 假设一次循环能处理 32 个字节

        // 预期的字节数据，手动计算的结果
        let expected1_bytes = vec![
            // 0x50, 0x00, // 3E 00 为MC协议的固定头
            0x00, // 00(网路编号) ：上位访问下位，固定00；
            0xFF, // FF(PLC编号) ：上位访问下位，固定FF；
            0xFF, 0x03, // 03(目标模块IO编号) ：上位访问下位，固定03；
            0x00, // 00(目标模块站号) ：上位访问下位，固定00；
            0x0C, 0x00, // 0x0C 为请求数据的长度
            0x10, 0x00, //
            0x01, 0x04, // 01 04 为读取命令
            0x00, 0x00, // 按字读取，如果按位读取则为 0x01 0x00
            0x00, 0x00, 0x00, // 起始地址 50
            0xA8, // 软元件代码9C为X为软元件代码
            0x84, 0x03, // 读取的软元件点数
        ];

        // 验证第一个字节数组是否与预期匹配bytes
        assert_eq!(
            result[0].to_vec(),
            expected1_bytes,
            "The first byte block does not match the expected bytes"
        );

        // 打印调试信息
        println!("Generated bytes: {:?}", result[0].to_vec());
        println!("Expected bytes: {:?}", expected1_bytes);

        // 预期的字节数据，手动计算的结果
        let expected2_bytes = vec![
            // 0x50, 0x00, // 3E 00 为MC协议的固定头
            0x00, // 00(网路编号) ：上位访问下位，固定00；
            0xFF, // FF(PLC编号) ：上位访问下位，固定FF；
            0xFF, 0x03, // 03(目标模块IO编号) ：上位访问下位，固定03；
            0x00, // 00(目标模块站号) ：上位访问下位，固定00；
            0x0C, 0x00, // 0x0C 为请求数据的长度
            0x10, 0x00, //
            0x01, 0x04, // 01 04 为读取命令
            0x00, 0x00, // 按字读取，如果按位读取则为 0x01 0x00
            0x84, 0x03, 0x00, // 起始地址 50
            0xA8, // 软元件代码9C为X为软元件代码
            0x01, 0x00, // 读取的软元件点数
        ];

        // 验证第一个字节数组是否与预期匹配bytes
        assert_eq!(
            result[1].to_vec(),
            expected2_bytes,
            "The first byte block does not match the expected bytes"
        );

        // 打印调试信息
        println!("Generated2 bytes: {:?}", result[1].to_vec());
        println!("Expected2 bytes: {:?}", expected2_bytes);
    }

    #[test]
    fn test_write_bit_to_bytes() {
        // 构造一个 ReadBits 请求
        let request = Request::WriteBools(
            "X0".to_owned().into(),
            vec![true, false, true, true, true].into(),
        );

        // 调用 try_from，尝试将 Request 转换为 Bytes
        let result = Vec::try_from(request.clone()).unwrap();

        // 验证结果是否符合预期
        assert!(!result.is_empty(), "Result should not be empty");

        // 预期的字节数据，手动计算的结果
        let expected_bytes = vec![
            // 0x50, 0x00, // 3E 00 为MC协议的固定头
            0x00, // 00(网路编号) ：上位访问下位，固定00；
            0xFF, // FF(PLC编号) ：上位访问下位，固定FF；
            0xFF, 0x03, // 03(目标模块IO编号) ：上位访问下位，固定03；
            0x00, // 00(目标模块站号) ：上位访问下位，固定00；
            0x0F, 0x00, // 0x0C 为请求数据的长度
            0x10, 0x00, //
            0x01, 0x14, // 01 04 为读取命令
            0x01, 0x00, // 按字读取，如果按位读取则为 0x01 0x00
            0x00, 0x00, 0x00, // 起始地址 50
            0x9C, // 软元件代码9C为X为软元件代码
            0x05, 0x00, // 读取的软元件点数
            0x10, 0x11, 0x10,
        ];

        // 验证第一个字节数组是否与预期匹配bytes
        assert_eq!(
            result[0].to_vec(),
            expected_bytes,
            "The first byte block does not match the expected bytes"
        );

        // 打印调试信息
        println!("Generated bytes: {:?}", result[0].to_vec());
        println!("Expected bytes: {:?}", expected_bytes);
    }

    #[test]
    fn test_write_words_to_bytes() {
        // 构造一个 ReadWords 请求
        let request =
            Request::WriteU16s("D0".to_owned().into(), vec![1, 2, 3, 4, 5].into());

        // 调用 try_from，尝试将 Request 转换为 Bytes
        let result = Vec::try_from(request.clone()).unwrap();

        // 预期的字节数据，手动计算的结果
        let expected_bytes = vec![
            // 0x50, 0x00,
            0x00, 0xFF, 0xFF, 0x03, 0x00, 0x16, 0x00, // 0x0C 为请求数据的长度
            0x10, 0x00, //, 0x00, 0x01,
            0x01, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0xA8, 0x05, 0x00, 0x01, 0x00, 0x02, 0x00,
            0x03, 0x00, 0x04, 0x00, 0x05, 0x00,
        ];
        // 验证第一个字节数组是否与预期匹配bytes
        assert_eq!(
            result[0].to_vec(),
            expected_bytes,
            "The first byte block does not match the expected bytes"
        );

        // 打印调试信息
        println!("Generated bytes: {:?}", result[0].to_vec());
        println!("Expected bytes: {:?}", expected_bytes);
    }

    #[test]
    fn test_bools_to_packed_bytes() {
        let bits = vec![true, false, true, true, true];
        let result = bools_to_bytes(&bits);

        // 打印为十六进制格式
        print!("Result in hex: [");
        for (i, byte) in result.iter().enumerate() {
            if i > 0 {
                print!(", ");
            }
            print!("0x{:02X}", byte); // 使用 `:02X` 确保补齐两位大写十六进制
        }
        println!("]");
    }
}
