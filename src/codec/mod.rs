use std::{
    convert::TryFrom,
    io::{self, Cursor, Error, ErrorKind},
};

use byteorder::{BigEndian, ReadBytesExt as _};

use crate::{
    bytes::{BufMut, Bytes, BytesMut},
    frame::*,
    header::RequestHeader,
};
#[allow(clippy::cast_possible_truncation)]
fn u16_len(len: usize) -> u16 {
    // This type conversion should always be safe, because either
    // the caller is responsible to pass a valid usize or the
    // possible values are limited by the protocol.
    debug_assert!(len <= u16::MAX.into());
    len as u16
}

#[allow(clippy::cast_possible_truncation)]
fn u8_len(len: usize) -> u8 {
    // This type conversion should always be safe, because either
    // the caller is responsible to pass a valid usize or the
    // possible values are limited by the protocol.
    debug_assert!(len <= u8::MAX.into());
    len as u8
}

impl<'a> TryFrom<Request<'a>> for Bytes {
    type Error = Error;

    #[allow(clippy::panic_in_result_fn)] // Intentional unreachable!()
    fn try_from(req: Request<'a>) -> Result<Bytes, Self::Error> {
        use crate::frame::Request::*;
        let header = RequestHeader::new();
        let cnt = request_byte_count(&req, header.len());
        let mut data = BytesMut::with_capacity(cnt);
        data.put_slice(header.bytes());
        match req {
            ReadBits(address, quantity, code) | ReadWords(address, quantity, code) => {
                // 断言确保 address 在 u24 的范围内
                assert!(address <= 0xFFFFFF, "Address out of range for u24");
                data.put_u16_le((address & 0xFFFF) as u16);
                data.put_u8((address >> 16) as u8); // 高位字节 |
                data.put_u8(code as u8);
                data.put_u16_le(quantity);
            }
            WriteMultipleBits(address, bits, code) => {}
            WriteMultipleWords(address, words, code) => {}
        }

        Ok(data.freeze())
    }
}

fn request_byte_count(req: &Request<'_>, header_len: usize) -> usize {
    use crate::frame::Request::*;
    match *req {
        ReadBits(_, _, _) | ReadWords(_, _, _) => header_len + 10,
        WriteMultipleBits(_, ref bits, _) => header_len + REQUEST_BYTE_LAST_LEN + bits.len(),
        WriteMultipleWords(_, ref words, _) => header_len + REQUEST_BYTE_LAST_LEN + words.len() * 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use std::convert::TryFrom;

    #[test]
    fn test_read_bits_to_bytes() {
        // 构造一个 ReadBits 请求
        let request = Request::ReadBits(0, 10, SoftElementCode::X);

        // 调用 try_from，尝试将 Request 转换为 Bytes
        let result = Bytes::try_from(request);

        // 验证转换成功
        assert!(result.is_ok());

        // 获取转换后的字节数据
        let bytes = result.unwrap();

        // 打印字节数组的十六进制表示
        print!("Bytes in hex: ");
        for byte in bytes.iter() {
            print!("{:02X} ", byte);
        }

        // // 预期的字节数据，手动计算的结果
        // let expected_bytes = vec![
        //     0x50, 0x00, 0x00, 0xFF, 0xFF, 0x03, 0x00, 0x00, 0x0C, 0x00, 0x10, 0x00, 0x56, 0x34,
        //     0x12, 0x9C, 0x0A, 0x00,
        // ];

        // // 比较生成的字节与预期结果是否相等
        // assert_eq!(bytes.as_ref(), expected_bytes.as_slice());
    }

    #[test]
    fn test_read_words_to_bytes() {
        // 构造一个 ReadWords 请求
        let request = Request::ReadWords(0x654321, 20, SoftElementCode::D);

        // 调用 try_from，尝试将 Request 转换为 Bytes
        let result = Bytes::try_from(request);

        // 验证转换成功
        assert!(result.is_ok());

        // 获取转换后的字节数据
        let bytes = result.unwrap();

        // 预期的字节数据，手动计算的结果
        let expected_bytes = vec![
            0x50, 0x00, 0x00, 0xFF, 0xFF, 0x03, 0x00, 0x00, 0x0C, 0x00, 0x10, 0x00, 0x21, 0x43,
            0x65, 0xA8, 0x14, 0x00,
        ];

        // 比较生成的字节与预期结果是否相等
        assert_eq!(bytes.as_ref(), expected_bytes.as_slice());
    }
}
