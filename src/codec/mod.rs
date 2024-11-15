use std::{convert::TryFrom, io::Cursor};

use byteorder::{ByteOrder, LittleEndian, ReadBytesExt as _};
use bytes::Buf;

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
        let cnt = request_byte_count(&req, header.len());

        // 获取通用的地址、代码和进制数
        let (address, quantity_or_len) = match req {
            ReadBits(ref address, quantity) => {
                let adjusted_quantity = (quantity as f64 / 16.0).ceil() as u16;
                (address.clone(), adjusted_quantity)
            }
            ReadWords(ref address, quantity) => (address.clone(), quantity),
            WriteMultipleBits(ref address, ref bits) => {
                (address.clone(), bits.len().try_into().unwrap())
            }
            WriteMultipleWords(ref address, ref words) => {
                (address.clone(), words.len().try_into().unwrap())
            }
        };

        let mut results = Vec::new();

        let (u32_number, code) = parse_address_and_get_instruction_code(&address)?;

        let mut current_len = quantity_or_len;

        let mut current_address = u32_number;

        while current_len > 0 {
            let len = current_len.min(LIMIT);
            let mut data = BytesMut::with_capacity(cnt);
            data.put_slice(header.bytes());
            data.put_slice(&req.function_code().value());
            request_command(&mut data, current_address, code, len);
            current_address += len as u32;
            current_len = current_len.saturating_sub(len);

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
            Request::ReadBits(_, quantity) => {
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
            
                Ok(Response::ReadBits(bits))
            }       
            Request::ReadWords(_, quantity) => {
                let mut words = Vec::with_capacity(quantity as usize);
                for _ in 0..quantity {
                    // 读取小端字节序的 u16 值并放入 words 向量g
                    let word = final_rdr.read_u16::<LittleEndian>()?;
                    words.push(word);
                }

                Ok(Response::ReadWords(words))
            }
            Request::WriteMultipleBits(_, _) => Ok(Response::WriteMultipleBits()),
            Request::WriteMultipleWords(_, _) => Ok(Response::WriteMultipleWords()),
        }
    }
}

fn request_byte_count(req: &Request<'_>, header_len: usize) -> usize {
    use crate::frame::Request::*;
    match *req {
        ReadBits(_, _) | ReadWords(_, _) => header_len + REQUEST_BYTE_LAST_LEN,
        WriteMultipleBits(_, ref bits) => header_len + REQUEST_BYTE_LAST_LEN + (bits.len() + 1) / 2,
        WriteMultipleWords(_, ref words) => header_len + REQUEST_BYTE_LAST_LEN + words.len() * 2,
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

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use std::convert::TryFrom;

    // #[test]
    // fn test_read_bits_to_bytes() {
    //     // 构造一个 ReadBits 请求
    //     let request = Request::ReadBits("X0".to_owned().into(), 10);

    //     // 调用 try_from，尝试将 Request 转换为 Bytes
    //     let result = Bytes::try_from(request);

    //     // 验证转换成功
    //     assert!(result.is_ok());

    //     // 获取转换后的字节数据
    //     let bytes = result.unwrap();

    //     // 预期的字节数据，手动计算的结果
    //     let expected_bytes = vec![
    //         0x50, 0x00, // 3E 00 为MC协议的固定头
    //         0x00, // 00(网路编号) ：上位访问下位，固定00；
    //         0xFF, // FF(PLC编号) ：上位访问下位，固定FF；
    //         0xFF, 0x03, // 03(目标模块IO编号) ：上位访问下位，固定03；
    //         0x00, // 00(目标模块站号) ：上位访问下位，固定00；
    //         0x0C, 0x00, // 0x0C 为请求数据的长度
    //         0x10, 0x00, //
    //         0x01, 0x04, // 01 04 为读取命令
    //         0x01, 0x00, // 按字读取，如果按位读取则为 0x01 0x00
    //         0x00, 0x00, 0x00, // 起始地址 50
    //         0x9C, // 软元件代码9C为X为软元件代码
    //         0x0A, 0x00, // 读取的软元件点数
    //     ];

    //     // 比较生成的字节与预期结果是否相等
    //     assert_eq!(bytes.as_ref(), expected_bytes.as_slice());
    // }

    #[test]
    fn test_read_words_to_bytes() {
        // 构造一个 ReadWords 请求
        let request = Request::ReadWords("D0".to_owned().into(), 9000);

        // 调用 try_from，尝试将 Request 转换为 Bytes
        let result = Vec::try_from(request.clone()).unwrap();
        // 验证结果是否符合预期
        assert!(!result.is_empty(), "Result should not be empty");

        // // 检查生成的 Bytes 的数量是否正确
        // // 假设 LIMIT 是你定义的分块逻辑的限制大小
        // let expected_chunks = (20 + LIMIT - 1) / LIMIT; // 计算需要多少块
        // assert_eq!(result.len(), expected_chunks, "Unexpected number of chunks");

        // 验证每个 Bytes 的结构
        for (i, chunk) in result.iter().enumerate() {
            println!("Chunk {}: {:?}", i, chunk);
            // 添加对 chunk 的具体内容验证逻辑，例如 header 或数据内容
            // 假设 `header.bytes()` 和 `req.function_code().value()` 已知
            let header_length = RequestHeader::new().bytes().len();
            assert!(chunk.len() >= header_length, "Chunk too short");
        }
    }

    // #[test]
    // fn test_write_bit_to_bytes() {
    //     // 构造一个 ReadBits 请求
    //     let request = Request::WriteMultipleBits(
    //         0,
    //         vec![true, false, true, true, true].into(),
    //         SoftElementCode::X,
    //     );

    //     // 调用 try_from，尝试将 Request 转换为 Bytes
    //     let result = Bytes::try_from(request);

    //     // 验证转换成功
    //     assert!(result.is_ok());

    //     // 获取转换后的字节数据
    //     let bytes = result.unwrap();

    //     // 预期的字节数据，手动计算的结果
    //     let expected_bytes = vec![
    //         0x50, 0x00, // 3E 00 为MC协议的固定头
    //         0x00, // 00(网路编号) ：上位访问下位，固定00；
    //         0xFF, // FF(PLC编号) ：上位访问下位，固定FF；
    //         0xFF, 0x03, // 03(目标模块IO编号) ：上位访问下位，固定03；
    //         0x00, // 00(目标模块站号) ：上位访问下位，固定00；
    //         0x0F, 0x00, // 0x0C 为请求数据的长度
    //         0x10, 0x00, //
    //         0x01, 0x14, // 01 04 为读取命令
    //         0x01, 0x00, // 按字读取，如果按位读取则为 0x01 0x00
    //         0x00, 0x00, 0x00, // 起始地址 50
    //         0x9C, // 软元件代码9C为X为软元件代码
    //         0x05, 0x00, // 读取的软元件点数
    //         0x10, 0x11, 0x10,
    //     ];

    //     // 比较生成的字节与预期结果是否相等
    //     assert_eq!(bytes.as_ref(), expected_bytes.as_slice());
    // }

    // #[test]
    // fn test_write_words_to_bytes() {
    //     // 构造一个 ReadWords 请求
    //     let request =
    //         Request::WriteMultipleWords(0x654321, vec![1, 2, 3, 4, 5].into(), SoftElementCode::D);

    //     // 调用 try_from，尝试将 Request 转换为 Bytes
    //     let result = Bytes::try_from(request);

    //     // 验证转换成功
    //     assert!(result.is_ok());

    //     // 获取转换后的字节数据
    //     let bytes = result.unwrap();

    //     // 预期的字节数据，手动计算的结果
    //     let expected_bytes = vec![
    //         0x50, 0x00, 0x00, 0xFF, 0xFF, 0x03, 0x00, 0x16,
    //         0x00, // 0x0C 为请求数据的长度
    //         0x10, 0x00, //, 0x00, 0x01,
    //         0x01, 0x14, 0x00, 0x00, 0x21, 0x43, 0x65, 0xA8, 0x05, 0x00, 0x01, 0x00, 0x02, 0x00,
    //         0x03, 0x00, 0x04, 0x00, 0x05, 0x00,
    //     ];

    //     // 比较生成的字节与预期结果是否相等
    //     assert_eq!(bytes.as_ref(), expected_bytes.as_slice());
    // }
}
