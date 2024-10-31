use std::{
    convert::TryFrom,
    io::{Cursor, Error},
};

use byteorder::{LittleEndian, ReadBytesExt as _};
use bytes::Buf;

use crate::{
    bytes::{BufMut, Bytes, BytesMut},
    frame::*,
    header::{RequestHeader, ResponseHeader},
};

impl<'a> TryFrom<Request<'a>> for Bytes {
    type Error = Error;

    #[allow(clippy::panic_in_result_fn)] // Intentional unreachable!()
    fn try_from(req: Request<'a>) -> Result<Bytes, Error> {
        use crate::frame::Request::*;
        let header = RequestHeader::new();
        let cnt = request_byte_count(&req, header.len());
        let mut data = BytesMut::with_capacity(cnt);
        data.put_slice(header.bytes());
        data.put_slice(&req.function_code().value());

        // 获取通用的地址、代码和进制数
        let (address, quantity_or_len) = match req {
            ReadBits(ref address, quantity) | ReadWords(ref address, quantity) => {
                (address.clone(), quantity)
            }
            WriteMultipleBits(ref address, ref bits) => {
                (address.clone(), bits.len().try_into().unwrap())
            }
            WriteMultipleWords(ref address, ref words) => {
                (address.clone(), words.len().try_into().unwrap())
            }
        };

        let (prefix, number) = split_address(&address).unwrap();
        let (code, number_base) = find_instruction_code(prefix).unwrap();
        let u32_number = convert_to_base(number, number_base).unwrap();

        // 通用请求命令
        request_command(&mut data, u32_number, code, quantity_or_len);

        // 根据不同请求类型完成额外数据写入
        match req {
            ReadBits(_, _) | ReadWords(_, _) => {}
            WriteMultipleBits(_, ref bits) => {
                // 将 `bits` 中的每两个 bit 组合成一个字节并写入
                for chunk in bits.chunks(2) {
                    let byte = (chunk[0] as u8) << 4 | chunk.get(1).map_or(0, |&bit| bit as u8);
                    data.put_u8(byte);
                }
            }
            WriteMultipleWords(_, ref words) => {
                // 将 words 中每个 u16 值以小端序写入 data
                for &word in words.iter() {
                    data.put_u16_le(word);
                }
            }
        }
        let len_bytes = ((data.len() - header.len() + 2) as u16).to_le_bytes();
        data[7..9].copy_from_slice(&len_bytes);
        Ok(data.freeze())
    }
}

impl TryFrom<(Bytes, Request<'_>)> for Response {
    type Error = Error;
    fn try_from((bytes, req): (Bytes, Request)) -> Result<Self, Self::Error> {
        let header = ResponseHeader::new();

        // // 使用 matches 方法检查帧头是否匹配
        // if !header.matches(&bytes) {
        //     return Err(Error::new(ErrorKind::InvalidData, "帧头不匹配"));
        // }

        let mut rdr = Cursor::new(&bytes[header.bytes().len()..]); // 跳过帧头

        // // 使用 byteorder 库的小端读取方法
        // let first_byte = rdr.read_u16::<LittleEndian>()?;
        // if first_byte != 0x00 {
        //     return Err(Error::new(ErrorKind::InvalidData, "第一个字节不是 0x00"));
        // }

        // 根据请求类型解析响应数据
        match req {
            Request::ReadBits(_, quantity) => {
                let mut bits = Vec::with_capacity(quantity as usize * 2);
                while bits.len() < quantity as usize && rdr.remaining() > 0 {
                    let byte = rdr.get_u8();

                    // 直接解析高 4 位和低 4 位为布尔值，并添加到 bits 中
                    bits.push((byte >> 4) & 1 != 0);
                    if bits.len() < quantity as usize {
                        bits.push((byte & 1) != 0);
                    }
                }

                Ok(Response::ReadBits(bits))
            }
            Request::ReadWords(_, quantity) => {
                let mut words = Vec::with_capacity(quantity as usize);
                for _ in 0..quantity {
                    // 读取小端字节序的 u16 值并放入 words 向量g
                    let word = rdr.read_u16::<LittleEndian>()?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use std::convert::TryFrom;

    #[test]
    fn test_read_bits_to_bytes() {
        // 构造一个 ReadBits 请求
        let request = Request::ReadBits("X0".to_owned().into(), 10);

        // 调用 try_from，尝试将 Request 转换为 Bytes
        let result = Bytes::try_from(request);

        // 验证转换成功
        assert!(result.is_ok());

        // 获取转换后的字节数据
        let bytes = result.unwrap();

        // 预期的字节数据，手动计算的结果
        let expected_bytes = vec![
            0x50, 0x00, // 3E 00 为MC协议的固定头
            0x00, // 00(网路编号) ：上位访问下位，固定00；
            0xFF, // FF(PLC编号) ：上位访问下位，固定FF；
            0xFF, 0x03, // 03(目标模块IO编号) ：上位访问下位，固定03；
            0x00, // 00(目标模块站号) ：上位访问下位，固定00；
            0x0C, 0x00, // 0x0C 为请求数据的长度
            0x10, 0x00, //
            0x01, 0x04, // 01 04 为读取命令
            0x01, 0x00, // 按字读取，如果按位读取则为 0x01 0x00
            0x00, 0x00, 0x00, // 起始地址 50
            0x9C, // 软元件代码9C为X为软元件代码
            0x0A, 0x00, // 读取的软元件点数
        ];

        // 比较生成的字节与预期结果是否相等
        assert_eq!(bytes.as_ref(), expected_bytes.as_slice());
    }

    #[test]
    fn test_read_words_to_bytes() {
        // 构造一个 ReadWords 请求
        let request = Request::ReadWords("D100".to_owned().into(), 20);

        // 调用 try_from，尝试将 Request 转换为 Bytes
        let result = Bytes::try_from(request);

        // 验证转换成功
        assert!(result.is_ok());

        // 获取转换后的字节数据
        let bytes = result.unwrap();

        // 预期的字节数据，手动计算的结果
        let expected_bytes = vec![
            0x50, 0x00, 0x00, 0xFF, 0xFF, 0x03, 0x00, 0x0C,
            0x00, // 0x0C 为请求数据的长度
            0x10, 0x00, //, 0x00, 0x01,
            0x01, 0x04, 0x00, 0x00, 0x64, 0x00, 0x00, 0xA8, 0x14, 0x00,
        ];

        // 比较生成的字节与预期结果是否相等
        assert_eq!(bytes.as_ref(), expected_bytes.as_slice());
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
