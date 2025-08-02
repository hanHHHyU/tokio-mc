use std::{
    borrow::Cow,
    convert::TryFrom,
    io::{Cursor, Read},
};

use byteorder::{ByteOrder, LittleEndian, ReadBytesExt as _};
use log;

use crate::{
    bytes::{BufMut, Bytes, BytesMut},
    frame::*,
    header::RequestHeader,
    Error,
};
pub mod tcp;

/// 优化的bool到字节转换，使用预分配和更高效的位操作
#[inline]
pub fn bools_to_bytes(bools: &[bool]) -> Vec<u8> {
    let capacity = (bools.len() + 1) / 2;
    let mut result = Vec::with_capacity(capacity);

    let chunks = bools.chunks_exact(2);
    let remainder = chunks.remainder();

    // 处理成对的bool值
    for chunk in chunks {
        result.push((chunk[0] as u8) << 4 | (chunk[1] as u8));
    }

    // 处理剩余的单个bool值
    if !remainder.is_empty() {
        result.push((remainder[0] as u8) << 4);
    }

    result
}

/// 优化的字节到bool转换，预分配确切大小
#[inline]
pub fn bytes_to_bools(bytes: &[u8]) -> Vec<bool> {
    let mut result = Vec::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        result.push((byte >> 4) & 0x01 != 0);
        result.push(byte & 0x01 != 0);
    }
    result
}
/// 客户端编码器 - 将 Request 编码为字节数据发送给服务端
pub struct ClientEncoder;

/// 服务端解码器 - 将客户端发送的字节数据解码为 Request  
pub struct ServerDecoder;

/// 客户端解码器 - 将服务端返回的字节数据解码为 Response
pub struct ClientDecoder;

impl ClientEncoder {
    /// 将 Request 编码为字节数据发送给服务端
    pub fn encode<'a>(req: Request<'a>) -> Result<Vec<Bytes>, Error> {
        // 调用现有的 TryFrom 实现
        Vec::try_from(req)
    }
}

impl ServerDecoder {
    /// 将客户端发送的字节数据解码为 Request
    pub fn decode(bytes: Bytes) -> Result<Request<'static>, Error> {
        // 调用现有的 TryFrom 实现
        Request::try_from(bytes)
    }
}

impl ClientDecoder {
    /// 将服务端返回的字节数据解码为 Response
    pub fn decode(bytes: Vec<Bytes>, req: Request<'_>) -> Result<Response, Error> {
        // 调用现有的 TryFrom 实现
        Response::try_from((bytes, req))
    }
}

// 客户端编码: Request -> Vec<Bytes> (客户端发送请求时使用)
impl<'a> TryFrom<Request<'a>> for Vec<Bytes> {
    type Error = Error;

    fn try_from(req: Request<'a>) -> Result<Vec<Bytes>, Error> {
        use crate::frame::Request::*;

        let (address, quantity_or_len, write_cursor) = match req {
            ReadU8s(ref address, quantity) => (address.clone(), quantity, None),
            WriteU8s(ref address, ref u8s) => {
                let cursor = Cursor::new(Cow::Owned(u8s.to_vec()));
                (
                    address.clone(),
                    ((u8s.len() as f32) / 2.0).round() as u32,
                    Some(WriteCursor::U8s(cursor)),
                )
            }
            ReadBits(ref address, quantity) => (address.clone(), quantity, None),
            WriteBits(ref address, ref bits) => {
                let bytes = bools_to_bytes(bits);
                let cursor = Cursor::new(Cow::Owned(bytes));
                (
                    address.clone(),
                    bits.len() as u32,
                    Some(WriteCursor::Bits(cursor)),
                )
            }
        };

        enum WriteCursor {
            U8s(Cursor<Cow<'static, [u8]>>),
            Bits(Cursor<Cow<'static, [u8]>>),
        }

        let mut results = Vec::new();
        let (u32_number, code) = parse_address_and_get_instruction_code(&address)?;
        let mut current_len = quantity_or_len;
        let mut current_address = u32_number;
        let header = RequestHeader::new();

        while current_len > 0 {
            let len = current_len.min(LIMIT) as u16;

            let mut data = match write_cursor {
                Some(WriteCursor::U8s(_)) => BytesMut::with_capacity(
                    header.len() + REQUEST_BYTE_LAST_LEN + (len * 2) as usize,
                ),
                Some(WriteCursor::Bits(_)) => {
                    BytesMut::with_capacity(header.len() + REQUEST_BYTE_LAST_LEN + len as usize)
                }
                None => BytesMut::with_capacity(header.len() + REQUEST_BYTE_LAST_LEN),
            };

            data.put_slice(header.bytes());
            data.put_slice(&req.function_code().value());
            request_command(&mut data, current_address, code, len);

            if let Some(write_cursor) = &write_cursor {
                match write_cursor {
                    WriteCursor::U8s(cursor) => {
                        let mut write_iter = cursor.get_ref().iter().cloned();
                        for _ in 0..len * 2 {
                            if let Some(value) = write_iter.next() {
                                data.put_u8(value);
                            }
                        }
                    }
                    WriteCursor::Bits(cursor) => {
                        // bit写入时，每个字节包含实际数据
                        let bytes_data = cursor.get_ref();
                        for &byte_val in bytes_data.iter() {
                            data.put_u8(byte_val);
                        }
                    }
                }
            }

            let length = (data.len() - header.len() + 2) as u16;
            LittleEndian::write_u16(&mut data[header.len() - 4..header.len() - 2], length);

            current_address += len as u32;
            current_len = current_len.saturating_sub(len as u32);
            results.push(data.freeze());
        }

        Ok(results)
    }
}

// 客户端解码: (Vec<Bytes>, Request) -> Response (客户端解析服务端响应时使用)
impl TryFrom<(Vec<Bytes>, Request<'_>)> for Response {
    type Error = Error;
    fn try_from((bytes, req): (Vec<Bytes>, Request)) -> Result<Self, Error> {
        log::debug!("=== Client received response from server ===");
        for (i, byte_chunk) in bytes.iter().enumerate() {
            log::debug!("Response chunk {}: {:02X?}", i, byte_chunk.as_ref());
        }

        let mut data = Vec::new();

        // for byte in &bytes {
        //     check_response(&byte)?;
        //     data.extend_from_slice(&byte[2..]);
        // }

        for (i, byte) in bytes.iter().enumerate() {
            // // 确保至少有 2 字节结束码
            // if byte.len() < 2 {
            //     return Err(Error::Protocol(format!("Response too short: {:?}", byte)));
            // }

            // // 检查结束码是否为 0x0000
            // let end_code = u16::from_le_bytes([byte[0], byte[1]]);
            // if end_code != 0x0000 {
            //     return Err(Error::PlcErrorCode(end_code));
            // }

            // 提取结束码之后的有效数据（注意：不是 byte[2..] 是跳过 end_code）
            data.extend_from_slice(&byte[2..]);
        }

        log::debug!("Response data after processing: {:02X?}", data);

        let final_rdr = Cursor::new(data);

        match req {
            Request::ReadU8s(_, _) => Ok(Response::ReadU8s(final_rdr.get_ref().to_vec())),
            Request::WriteU8s(_, _) => Ok(Response::WriteU8s()),
            Request::ReadBits(_, _) => {
                let bytes = final_rdr.get_ref().to_vec();
                let bits = bytes_to_bools(&bytes);
                Ok(Response::ReadBits(bits))
            }
            Request::WriteBits(_, _) => Ok(Response::WriteBits()),
        }
    }
}

// impl TryFrom<(Vec<Bytes>)> for Request {
//     type Error = Error;
//     fn try_from((bytes): (Vec<Bytes>)) -> Result<Self, Error> {
//         let mut cursor = Cursor::new(bytes);
//         let mut instruction_code = [0u8; 4];
//     }
// }

// 服务端解码: Bytes -> Request (服务端解析客户端请求时使用)
impl<'a> TryFrom<Bytes> for Request<'a> {
    type Error = Error;

    fn try_from(bytes: Bytes) -> Result<Self, Error> {
        let mut cursor = Cursor::new(bytes);

        let _len = cursor.read_u16::<LittleEndian>()? as usize;

        // 2. 跳过监视定时器 (2字节)
        cursor.read_u16::<LittleEndian>()?; // 跳过 [10, 00]

        // 打印cursor的数据
        log::debug!("Cursor data: {:?}", cursor.get_ref());

        let mut instruction_code = [0u8; 4];

        cursor.read_exact(&mut instruction_code)?;
        let function_code = FunctionCode::new(BytesMut::from(&instruction_code[..]))
            .ok_or_else(|| Error::Protocol(ProtocolError::InvalidFunctionCode(instruction_code)))?;

        let start_addr = cursor.read_u24::<LittleEndian>()?;
        let (prefix, number_base) = find_prefix_and_base_by_code(cursor.read_u8()?).unwrap();
        let quantity = cursor.read_u16::<LittleEndian>()? as u32;

        if quantity > LIMIT {
            return Err(Error::Protocol(ProtocolError::OutOfRange));
        }

        // 打印prefix
        log::debug!("Prefix: {}", prefix);
        // 打印number_base
        log::debug!("Number base: {:?}", number_base);

        // start_addr根据number_base转换为对应string格式
        let start_addr: String = match number_base {
            NumberBase::Decimal => format!("{}", start_addr),
            NumberBase::Hexadecimal => format!("{:X}", start_addr),
        };

        log::debug!("Start address (string): {}", start_addr);

        let address: Cow<'a, str> = format!("{}{}", prefix, start_addr).into();

        log::debug!("Raw instruction code: {:02X?}", instruction_code);
        log::debug!("Parsed function code: {:?}", function_code);
        log::debug!("Start address: {}", address);
        log::debug!("Raw quantity: {}", quantity);

        // 打印start_addr
        log::debug!("Start address (u32): {}", start_addr);

        match function_code {
            FunctionCode::ReadU8s => Ok(Request::ReadU8s(address, quantity)),
            FunctionCode::WriteU8s => {
                let u8s = cursor.get_ref()[cursor.position() as usize..].to_vec();
                log::debug!("Parsed U8s: {:?}", u8s);

                // if u8s.len() != quantity as usize {
                //     return Err(Error::Protocol(ProtocolError::OutOfRange));
                // }
                Ok(Request::WriteU8s(address, u8s.into()))
            }
            FunctionCode::ReadBits => Ok(Request::ReadBits(address, quantity)),
            FunctionCode::WriteBits => {
                let bytes = cursor.get_ref()[cursor.position() as usize..].to_vec();
                let bits = bytes_to_bools(&bytes);
                log::debug!("Parsed bits: {:?}", bits);
                Ok(Request::WriteBits(address, bits.into()))
            }
        }
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

// fn check_response(response_bytes: &[u8]) -> Result<(), Error> {
//     // let header_len = ResponseHeader::new().len();
//     // 获取响应字节缓冲区的前 `header_len` 字节，并提取最后两个字节
//     let last_two_bytes = &response_bytes[..2];
//     // 将最后两个字节转换为小端格式的 16 位整数
//     let last_two = LittleEndian::read_u16(last_two_bytes);

//     if let Some(error) = map_error_code(last_two) {
//         return Err(error.into());
//     }

//     Ok(())
// }

// fn reverse(bs: &mut [u8]) {
//     let len = bs.len();
//     for i in 0..len / 2 {
//         let num = i * 2;
//         let num2 = num + 1;

//         if num2 < len {
//             bs.swap(num, num2);
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn test_read_u8s_to_bytes() {
        let request = Request::ReadU8s("D0".to_owned().into(), 10);
        let result = Vec::try_from(request);
        assert!(result.is_ok());

        let bytes = result.unwrap();
        assert_eq!(bytes.len(), 1);

        let expected_bytes = vec![
            0x50, 0x00, 0x00, 0xFF, 0xFF, 0x03, 0x00, 0x0C, 0x00, 0x10, 0x00, 0x01, 0x04, 0x00,
            0x00, 0x00, 0x00, 0x00, 0xA8, 0x0A, 0x00,
        ];

        assert_eq!(
            bytes[0].to_vec(),
            expected_bytes,
            "The byte block does not match the expected bytes"
        );
    }

    #[test]
    fn test_write_u8s_to_bytes() {
        let data: Vec<u8> = vec![1, 2, 3, 4];
        let request = Request::WriteU8s("D0".to_owned().into(), data.clone().into());
        let result = Vec::try_from(request);
        assert!(result.is_ok());

        let bytes = result.unwrap();
        let mut expected_bytes = vec![
            0x50, 0x00, 0x00, 0xFF, 0xFF, 0x03, 0x00, 0x10, 0x00, 0x10, 0x00, 0x01, 0x14, 0x00,
            0x00, 0x00, 0x00, 0x00, 0xA8, 0x02, 0x00,
        ];

        expected_bytes.extend(data);

        assert_eq!(
            bytes[0].to_vec(),
            expected_bytes,
            "The byte block does not match the expected bytes"
        );
    }

    #[test]
    fn test_read_bits_to_bytes() {
        let request = Request::ReadBits("M0".to_owned().into(), 8);
        let result = Vec::try_from(request);
        assert!(result.is_ok());

        let bytes = result.unwrap();
        assert_eq!(bytes.len(), 1);

        let expected_bytes = vec![
            0x50, 0x00, 0x00, 0xFF, 0xFF, 0x03, 0x00, 0x0C, 0x00, 0x10, 0x00, 0x01, 0x04, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x90, 0x08, 0x00,
        ];

        assert_eq!(
            bytes[0].to_vec(),
            expected_bytes,
            "The ReadBits byte block does not match the expected bytes"
        );
    }

    #[test]
    fn test_read_bits_different_quantities() {
        // 测试读取1个bit
        let request = Request::ReadBits("M0".to_owned().into(), 1);
        let result = Vec::try_from(request);
        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert_eq!(bytes.len(), 1);

        // 测试读取16个bit
        let request = Request::ReadBits("M0".to_owned().into(), 16);
        let result = Vec::try_from(request);
        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert_eq!(bytes.len(), 1);

        // 测试读取大量bit（超过单个请求限制）
        let request = Request::ReadBits("M0".to_owned().into(), 2000);
        let result = Vec::try_from(request);
        assert!(result.is_ok());
        let bytes = result.unwrap();
        // 应该会分割成多个请求
        assert!(bytes.len() > 1);
    }

    #[test]
    fn test_read_bits_different_addresses() {
        // 测试不同的地址格式
        let addresses = vec!["M0", "M100", "M1000", "X0", "Y0"];
        for addr in addresses {
            let request = Request::ReadBits(addr.to_owned().into(), 8);
            let result = Vec::try_from(request);
            assert!(result.is_ok(), "Failed for address: {}", addr);
        }
    }

    #[test]
    fn test_write_bits_to_bytes() {
        let data: Vec<bool> = vec![true, false, true, false];
        let request = Request::WriteBits("M0".to_owned().into(), data.clone().into());
        let result = Vec::try_from(request);
        assert!(result.is_ok());

        let bytes = result.unwrap();
        // let bytes = result.unwrap();
        let mut expected_bytes = vec![
            0x50, 0x00, 0x00, 0xFF, 0xFF, 0x03, 0x00, 0x0E, 0x00, 0x10, 0x00, 0x01, 0x14, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x90, 0x04, 0x00,
        ];

        let len = (data.len() + 1) / 2 + 12;

        // 替换expected_odd_bytes的长度部分
        expected_bytes[7] = (len & 0xFF) as u8; // 低字节
        expected_bytes[8] = ((len >> 8) & 0xFF) as u8; // 高字节

        // 添加转换后的bit数据
        expected_bytes.extend(bools_to_bytes(&data));

        // 计算后续
        assert_eq!(
            bytes[0].to_vec(),
            expected_bytes,
            "The WriteBits byte block does not match the expected bytes"
        );

        // 对于奇数长度的bit数组，最后一个bit应该补0
        let odd_data: Vec<bool> = vec![true, false, true];
        let odd_request = Request::WriteBits("M0".to_owned().into(), odd_data.clone().into());
        let odd_result = Vec::try_from(odd_request);
        assert!(odd_result.is_ok());
        let odd_bytes = odd_result.unwrap();
        let mut expected_odd_bytes = vec![
            0x50, 0x00, 0x00, 0xFF, 0xFF, 0x03, 0x00, 0x0E, 0x00, 0x10, 0x00, 0x01, 0x14, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x90, 0x03, 0x00,
        ];

        // 0x0E, 0x00的部分是指令长度
        let len = (odd_data.len() + 1) / 2 + 12;

        // 替换expected_odd_bytes的长度部分
        expected_odd_bytes[7] = (len & 0xFF) as u8; // 低字节
        expected_odd_bytes[8] = ((len >> 8) & 0xFF) as u8; // 高字节

        // 添加转换后的bit数据
        expected_odd_bytes.extend(bools_to_bytes(&odd_data));

        // println!("Expected odd bytes: {:02X?}", expected_odd_bytes);

        // 计算后续
        assert_eq!(
            odd_bytes[0].to_vec(),
            expected_odd_bytes,
            "The WriteBits byte block for odd length does not match the expected bytes"
        );
    }

    #[test]
    fn test_write_bits_different_patterns() {
        // 测试全部为true的bit模式
        let data: Vec<bool> = vec![true, true, true, true];
        let request = Request::WriteBits("M0".to_owned().into(), data.clone().into());
        let result = Vec::try_from(request);
        assert!(result.is_ok());
        let expected_bit_bytes = bools_to_bytes(&data);
        assert_eq!(expected_bit_bytes, vec![0x11, 0x11]);

        // 测试全部为false的bit模式
        let data: Vec<bool> = vec![false, false, false, false];
        let request = Request::WriteBits("M0".to_owned().into(), data.clone().into());
        let result = Vec::try_from(request);
        assert!(result.is_ok());
        let expected_bit_bytes = bools_to_bytes(&data);
        assert_eq!(expected_bit_bytes, vec![0x00, 0x00]);

        // 测试交替模式
        let data: Vec<bool> = vec![true, false, true, false, true, false];
        let request = Request::WriteBits("M0".to_owned().into(), data.clone().into());
        let result = Vec::try_from(request);
        assert!(result.is_ok());
        let expected_bit_bytes = bools_to_bytes(&data);
        assert_eq!(expected_bit_bytes, vec![0x10, 0x10, 0x10]);

        // 为奇数长度的bit数组测试
        let data: Vec<bool> = vec![true, false, true];
        let request = Request::WriteBits("M0".to_owned().into(), data.clone().into());
        let result = Vec::try_from(request);
        assert!(result.is_ok());
        let expected_bit_bytes = bools_to_bytes(&data);
        assert_eq!(expected_bit_bytes, vec![0x10, 0x10]);
    }

    #[test]
    fn test_write_bits_odd_length() {
        // 测试奇数长度的bit数组
        let data: Vec<bool> = vec![true, false, true];
        let request = Request::WriteBits("M0".to_owned().into(), data.clone().into());
        let result = Vec::try_from(request);
        assert!(result.is_ok());

        let expected_bit_bytes = bools_to_bytes(&data);
        // 奇数长度应该是 [0x10, 0x10] (最后一个bit补0)
        assert_eq!(expected_bit_bytes, vec![0x10, 0x10]);
    }

    #[test]
    fn test_write_bits_large_data() {
        // 测试大量bit数据（会被分割成多个请求）
        let data: Vec<bool> = (0..2000).map(|i| i % 2 == 0).collect();
        let request = Request::WriteBits("M0".to_owned().into(), data.clone().into());
        let result = Vec::try_from(request);
        assert!(result.is_ok());

        let bytes = result.unwrap();
        // 应该会分割成多个请求
        assert!(bytes.len() > 1);

        // 验证每个请求都包含正确的bit数据
        for byte_chunk in &bytes {
            assert!(byte_chunk.len() > 21); // 至少包含header + bit数据
        }
    }

    #[test]
    fn test_write_bits_different_addresses() {
        let data: Vec<bool> = vec![true, false, true, false];
        let addresses = vec!["M0", "M100", "X0", "Y0"];

        for addr in addresses {
            let request = Request::WriteBits(addr.to_owned().into(), data.clone().into());
            let result = Vec::try_from(request);
            assert!(result.is_ok(), "Failed for address: {}", addr);

            let bytes = result.unwrap();
            assert_eq!(bytes.len(), 1);

            // 验证包含的bit数据是正确的
            let byte_vec = bytes[0].to_vec();
            let bit_data = &byte_vec[21..]; // bit数据从第21字节开始
            assert_eq!(bit_data, bools_to_bytes(&data));
        }
    }

    #[test]
    fn test_bools_to_bytes() {
        let bools = vec![true, false, true, false];
        let bytes = bools_to_bytes(&bools);
        assert_eq!(bytes, vec![0x10, 0x10]);

        let bools = vec![true, false, true];
        let bytes = bools_to_bytes(&bools);
        assert_eq!(bytes, vec![0x10, 0x10]);
    }

    #[test]
    fn test_bytes_to_bools() {
        // 测试 [0x10, 0x00] -> [true, false, false, false]
        let bytes = vec![0x10, 0x00];
        let bools = bytes_to_bools(&bytes);
        assert_eq!(bools, vec![true, false, false, false]);

        // 测试 [0x10, 0x10] -> [true, false, true, false]
        let bytes = vec![0x10, 0x10];
        let bools = bytes_to_bools(&bytes);
        assert_eq!(bools, vec![true, false, true, false]);
    }
}
