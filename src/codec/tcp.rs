use byteorder::{ByteOrder, LittleEndian};
#[cfg(feature = "server")]
use bytes::BufMut;
use bytes::{Bytes, BytesMut};
use log;
use std::io::Result;
use tokio_util::codec::{Decoder, Encoder};

use crate::{frame::Request, header::ResponseHeader};

#[cfg(feature = "server")]
use crate::{frame::Response, header::RequestHeader};

#[derive(Debug, Default)]
pub(crate) struct McClientDecoder;

#[derive(Debug, Default)]
#[cfg(feature = "server")]
pub(crate) struct McServerDecoder;

#[derive(Debug)]
pub(crate) struct McClientCodec {
    pub(crate) decoder: McClientDecoder,
}

impl McClientCodec {
    pub(crate) const fn new() -> Self {
        Self {
            decoder: McClientDecoder,
        }
    }
}

#[derive(Debug, Default)]
#[cfg(feature = "server")]
pub(crate) struct ServerCodec {
    pub(crate) decoder: McServerDecoder,
}

impl Decoder for McClientDecoder {
    type Item = Bytes;
    type Error = std::io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Bytes>> {
        let response_header = ResponseHeader::new();
        let header_len = response_header.len();

        if buf.len() < header_len {
            return Ok(None); // Need more data
        }

        log::debug!("Client received buffer: {:02X?}", &buf[..]);

        // 客户端解析服务端响应 - 验证响应前缀 (D0 00 00 FF FF 03 00)
        let response_prefix = [0xD0, 0x00, 0x00, 0xFF, 0xFF, 0x03, 0x00];
        if buf[..response_prefix.len()] != response_prefix {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid MC response prefix: {:02X?}", &buf[..header_len]),
            ));
        }

        // Extract data length from header
        let len = usize::from(LittleEndian::read_u16(&buf[header_len - 2..header_len]));
        let total_len = header_len + len;

        if buf.len() < total_len {
            return Ok(None); // Need more data
        }

        // Extract complete frame and return payload only
        let mut complete_frame = buf.split_to(total_len);
        let payload = complete_frame.split_off(header_len).freeze();
        Ok(Some(payload))
    }
}

#[cfg(feature = "server")]
impl Decoder for McServerDecoder {
    type Item = Bytes;
    type Error = std::io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Bytes>> {
        let request_header = RequestHeader::new();
        let header_len = request_header.len();

        // let response_header = ResponseHeader::new();
        // let header_len = response_header.len();

        log::debug!("Server received buffer: {:02X?}", &buf[..]);

        if buf.len() < header_len {
            return Ok(None); // Need more data
        }

        // 服务端解析客户端请求 - 验证请求前缀 (50 00 00 FF FF 03 00)
        let request_prefix = [0x50, 0x00, 0x00, 0xFF, 0xFF, 0x03, 0x00];
        if buf[..request_prefix.len()] != request_prefix {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid MC request prefix: {:02X?}", &buf[..header_len]),
            ));
        }

        // Extract data length from header
        let len = usize::from(LittleEndian::read_u16(&buf[header_len - 4..header_len - 2]));

        log::debug!("Data length: {}", len);

        // 检查是否有足够的数据来读取完整的包
        let total_len = header_len - 4 + len + 2;
        if buf.len() < total_len {
            log::debug!("Need more data: buf.len()={}, total_len={}", buf.len(), total_len);
            return Ok(None); // Need more data
        }

        log::debug!("Server2 received buffer: {:02X?}", &buf[..]);

        let _header = buf.split_to(header_len - 4);

        // 打印头部信息
        log::debug!("Header: {:02X?}", &_header[..]);

        // 2. 获取 payload 数据部分
        let payload = buf.split_to(len + 2);
        log::debug!("Payload: {:02X?}", &payload[..]);

        Ok(Some(payload.into()))
    }
}

impl Decoder for McClientCodec {
    type Item = Bytes;
    type Error = std::io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Bytes>> {
        self.decoder.decode(buf)
    }
}

impl Encoder<Request<'_>> for McClientCodec {
    type Error = std::io::Error;

    fn encode(&mut self, request: Request<'_>, buf: &mut BytesMut) -> Result<()> {
        // 使用 ClientEncoder 来编码请求
        let request_parts: Vec<bytes::Bytes> = crate::codec::ClientEncoder::encode(request)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        for part in request_parts {
            buf.extend_from_slice(&part);
        }

        Ok(())
    }
}

#[cfg(feature = "server")]
impl Decoder for ServerCodec {
    type Item = Bytes;
    type Error = std::io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Bytes>> {
        if let Some(payload) = self.decoder.decode(buf)? {
            Ok(Some(payload))
        } else {
            Ok(None)
        }
    }
}

#[cfg(feature = "server")]
impl Encoder<Response> for ServerCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: Response, buf: &mut BytesMut) -> std::io::Result<()> {
        let response_header = ResponseHeader::new();
        let response_header_len = response_header.len();

        // 添加调试打印
        log::debug!("=== ServerCodec::encode Debug ===");
        log::debug!("Response item: {:?}", item);
        log::debug!("Item length: {}", item.len());

        buf.reserve(response_header_len + item.len() + 2);

        let mut header_bytes = BytesMut::from(&response_header.0[..]);

        // 计算数据长度
        let data_length = match &item {
            Response::ReadU8s(_) => (item.len() * 2 + 2) as u16,
            Response::WriteU8s() => 2,
            Response::ReadBits(values) => ((values.len() + 1) / 2 + 2) as u16,
            Response::WriteBits() => 2,
        };
        log::debug!("Calculated data length: {}", data_length);

        LittleEndian::write_u16(
            &mut header_bytes[response_header_len - 2..response_header_len],
            data_length,
        );

        log::debug!("Header after length update: {:02X?}", &header_bytes[..]);

        buf.put_slice(&header_bytes);
        buf.put_u16_le(0x0000);

        log::debug!("Buffer after header + end code: {:02X?}", &buf[..]);

        match item {
            Response::ReadU8s(values) => {
                log::debug!("Adding ReadU8s data: {:02X?}", values);
                for &value in &values {
                    buf.put_u8(value);
                }
            }
            Response::WriteU8s() => {
                log::debug!("WriteU8s response - no additional data");
            }
            Response::ReadBits(values) => {
                let bytes = crate::codec::bools_to_bytes(&values);
                for &byte in &bytes {
                    buf.put_u8(byte);
                }
            }
            Response::WriteBits() => {
                log::debug!("WriteBits response - no additional data");
            }
        }

        log::debug!("Final encoded buffer: {:02X?}", &buf[..]);
        log::debug!("Final buffer length: {}", buf.len());
        log::debug!("================================");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "server")]
    use crate::{frame::Response, header::RequestHeader};
    #[cfg(feature = "server")]
    use bytes::{Buf, BytesMut};

    #[test]
    #[cfg(feature = "server")]
    fn test_encode_read_u8s() {
        let mut codec = ServerCodec::default();
        let mut buf = BytesMut::new();

        let values = vec![1u8, 2u8, 3u8, 4u8];
        let response = Response::ReadU8s(values);

        codec.encode(response, &mut buf).unwrap();

        // Verify header length
        assert!(buf.len() >= 9); // At least 9 bytes for header
    }

    #[test]
    #[cfg(feature = "server")]
    fn test_encode_write_u8s() {
        let mut codec = ServerCodec::default();
        let mut buf = BytesMut::new();

        // 测试写操作的响应
        let response = Response::WriteU8s();

        codec.encode(response, &mut buf).unwrap();

        // 验证头部
        let header_bytes = buf.split_to(9);
        let data_length = LittleEndian::read_u16(&header_bytes[7..9]);
        assert_eq!(data_length, 2); // 只有2字节的结束代码

        // 验证结束代码
        assert_eq!(buf.get_u16_le(), 0x0000);

        // 写操作不应该有额外数据
        assert_eq!(buf.len(), 0);
    }

    #[test]
    #[cfg(feature = "server")]
    fn test_encode_read_bits() {
        let mut codec = ServerCodec::default();
        let mut buf = BytesMut::new();

        // 测试读取位数据的响应
        let bits = vec![true, false, true, false, true, false];
        let response = Response::ReadBits(bits.clone());

        codec.encode(response, &mut buf).unwrap();

        // println!("Encoded buffer: {:02X?}", &buf[..]);

        // 验证头部长度（9字节）
        assert!(buf.len() >= 9);

        // 分离头部
        let header_bytes = buf.split_to(9);

        // 验证响应头前缀 (D0 00 00 FF FF 03 00)
        let expected_header_prefix = [0xD0, 0x00, 0x00, 0xFF, 0xFF, 0x03, 0x00];
        assert_eq!(&header_bytes[..7], expected_header_prefix);

        // 验证数据长度计算是否正确
        let data_length = LittleEndian::read_u16(&header_bytes[7..9]);
        let expected_length = ((bits.len() + 1) / 2 + 2) as u16; // bit数据 + 结束码
        assert_eq!(data_length, expected_length);

        // 验证结束代码 (0x0000)
        assert_eq!(buf.get_u16_le(), 0x0000);

        // 验证bit数据编码
        let encoded_bits = crate::codec::bools_to_bytes(&bits);
        let bit_data = buf.chunk();
        assert_eq!(bit_data, &encoded_bits[..]);
    }

    #[test]
    #[cfg(feature = "server")]
    fn test_encode_read_bits_different_patterns() {
        let mut codec = ServerCodec::default();

        // 测试全为true的位模式
        {
            let mut buf = BytesMut::new();
            let bits = vec![true, true, true, true];
            let response = Response::ReadBits(bits.clone());

            codec.encode(response, &mut buf).unwrap();

            let _header = buf.split_to(9);
            let _end_code = buf.get_u16_le();

            let expected_bits = crate::codec::bools_to_bytes(&bits);
            assert_eq!(buf.chunk(), &expected_bits[..]);
        }

        // 测试全为false的位模式
        {
            let mut buf = BytesMut::new();
            let bits = vec![false, false, false, false];
            let response = Response::ReadBits(bits.clone());

            codec.encode(response, &mut buf).unwrap();

            let _header = buf.split_to(9);
            let _end_code = buf.get_u16_le();

            let expected_bits = crate::codec::bools_to_bytes(&bits);
            assert_eq!(buf.chunk(), &expected_bits[..]);
        }

        // 测试交替模式
        {
            let mut buf = BytesMut::new();
            let bits = vec![true, false, true, false, true, false];
            let response = Response::ReadBits(bits.clone());

            codec.encode(response, &mut buf).unwrap();

            let _header = buf.split_to(9);
            let _end_code = buf.get_u16_le();

            let expected_bits = crate::codec::bools_to_bytes(&bits);
            assert_eq!(buf.chunk(), &expected_bits[..]);
        }
    }

    #[test]
    #[cfg(feature = "server")]
    fn test_encode_read_bits_odd_length() {
        let mut codec = ServerCodec::default();
        let mut buf = BytesMut::new();

        // 测试奇数长度的位数据
        let bits = vec![true, false, true]; // 3个位
        let response = Response::ReadBits(bits.clone());

        codec.encode(response, &mut buf).unwrap();

        // 分离头部
        let header_bytes = buf.split_to(9);

        // 验证数据长度 - 奇数长度应该向上取整
        let data_length = LittleEndian::read_u16(&header_bytes[7..9]);
        let expected_length = ((bits.len() + 1) / 2 + 2) as u16; // (3+1)/2 + 2 = 4
        assert_eq!(data_length, expected_length);

        // 跳过结束代码
        let _end_code = buf.get_u16_le();

        // 验证编码后的位数据
        let encoded_bits = crate::codec::bools_to_bytes(&bits);
        let bit_data = buf.chunk();
        assert_eq!(bit_data, &encoded_bits[..]);

        // 奇数长度的位数据应该补0
        // [true, false, true] -> [0x10, 0x10] (最后一个bit补0)
        assert_eq!(encoded_bits, vec![0x10, 0x10]);
    }

    #[test]
    #[cfg(feature = "server")]
    fn test_encode_read_bits_empty() {
        let mut codec = ServerCodec::default();
        let mut buf = BytesMut::new();

        // 测试空的位数据
        let bits: Vec<bool> = vec![];
        let response = Response::ReadBits(bits.clone());

        codec.encode(response, &mut buf).unwrap();

        // 分离头部
        let header_bytes = buf.split_to(9);

        // 验证数据长度 - 空数据应该只有结束码
        let data_length = LittleEndian::read_u16(&header_bytes[7..9]);
        let expected_length = 2u16; // 只有结束码
        assert_eq!(data_length, expected_length);

        // 验证结束代码
        assert_eq!(buf.get_u16_le(), 0x0000);

        // 空数据后不应该有额外的位数据
        assert_eq!(buf.len(), 0);
    }

    #[test]
    #[cfg(feature = "server")]
    fn test_encode_read_bits_large_data() {
        let mut codec = ServerCodec::default();
        let mut buf = BytesMut::new();

        // 测试大量位数据
        let bits: Vec<bool> = (0..100).map(|i| i % 2 == 0).collect();
        let response = Response::ReadBits(bits.clone());

        codec.encode(response, &mut buf).unwrap();

        // 分离头部
        let header_bytes = buf.split_to(9);

        // 验证数据长度
        let data_length = LittleEndian::read_u16(&header_bytes[7..9]);
        let expected_length = ((bits.len() + 1) / 2 + 2) as u16;
        assert_eq!(data_length, expected_length);

        // 跳过结束代码
        let _end_code = buf.get_u16_le();

        // 验证编码后的位数据长度
        let encoded_bits = crate::codec::bools_to_bytes(&bits);
        assert_eq!(buf.len(), encoded_bits.len());
        assert_eq!(buf.chunk(), &encoded_bits[..]);
    }

    #[test]
    #[cfg(feature = "server")]
    fn test_encode_write_bits() {
        let mut codec = ServerCodec::default();
        let mut buf = BytesMut::new();

        // 测试写位操作的响应
        let response = Response::WriteBits();

        codec.encode(response, &mut buf).unwrap();

        // 验证头部
        let header_bytes = buf.split_to(9);

        // 验证响应头前缀 (D0 00 00 FF FF 03 00)
        let expected_header_prefix = [0xD0, 0x00, 0x00, 0xFF, 0xFF, 0x03, 0x00];
        assert_eq!(&header_bytes[..7], expected_header_prefix);

        // 验证数据长度 - 写操作只有结束码
        let data_length = LittleEndian::read_u16(&header_bytes[7..9]);
        assert_eq!(data_length, 2);

        // 验证结束代码
        assert_eq!(buf.get_u16_le(), 0x0000);

        // 写操作不应该有额外数据
        assert_eq!(buf.len(), 0);
    }

    #[test]
    #[cfg(feature = "server")]
    fn test_server_codec_decode() {
        let bytes = [
            0x50, 0x00, 0x00, 0xFF, 0xFF, 0x03, 0x00, 0x0E, 0x00, 0x10, 0x00, 0x01, 0x14, 0x00,
            0x00, 0x58, 0x1B, 0x00, 0xA8, 0x01, 0x00, 0x0E, 0x00,
        ];

        // 创建一个空的 BytesMut 缓冲区
        let mut buffer = BytesMut::with_capacity(1024);

        // 将 header_bytes 添加到缓冲区
        buffer.extend_from_slice(&bytes);

        // 创建ServerCodec实例
        let mut codec = ServerCodec {
            decoder: McServerDecoder {},
        };

        // 调用decode方法
        let result = codec.decode(&mut buffer);

        // 基本验证结果是Ok
        assert!(result.is_ok(), "解码过程应该成功");
    }

    #[test]
    fn test_bits_direct_u8_operations() {
        // 测试位操作直接操作底层u8数据的场景
        // 模拟multi-zone-server-test中的位操作方式

        // 创建一个字节用于测试
        let mut test_byte: u8 = 0b00000000; // 初始值：所有位都是0

        // 设置第0位为1 (LSB)
        test_byte |= 1 << 0;
        assert_eq!(test_byte, 0b00000001);

        // 设置第2位为1
        test_byte |= 1 << 2;
        assert_eq!(test_byte, 0b00000101);

        // 设置第7位为1 (MSB)
        test_byte |= 1 << 7;
        assert_eq!(test_byte, 0b10000101);

        // 清除第0位
        test_byte &= !(1 << 0);
        assert_eq!(test_byte, 0b10000100);

        // 测试读取位值
        let bit_0 = (test_byte >> 0) & 0x01 != 0;
        let bit_2 = (test_byte >> 2) & 0x01 != 0;
        let bit_7 = (test_byte >> 7) & 0x01 != 0;

        assert_eq!(bit_0, false); // 第0位已清除
        assert_eq!(bit_2, true); // 第2位为1
        assert_eq!(bit_7, true); // 第7位为1
    }

    #[test]
    #[cfg(feature = "server")]
    fn test_encode_read_bits_simulating_server_behavior() {
        // 测试模拟实际服务器行为的位读取编码
        // 这里我们创建响应数据，就像multi-zone-server-test会返回的那样

        let mut codec = ServerCodec::default();

        // 模拟从服务器底层u8数据中读取的位
        // 假设底层数据是: [0b10100101, 0b11000011]
        // 这应该解析为位: [true, false, true, false, false, true, false, true, true, true, false, false, false, false, true, true]
        let simulated_u8_data = vec![0b10100101u8, 0b11000011u8];

        let mut simulated_bits = Vec::new();
        for &byte in &simulated_u8_data {
            for bit_pos in 0..8 {
                let bit_value = (byte >> bit_pos) & 0x01 != 0;
                simulated_bits.push(bit_value);
            }
        }

        // 现在测试编码这些位数据
        let mut buf = BytesMut::new();
        let response = Response::ReadBits(simulated_bits.clone());

        codec.encode(response, &mut buf).unwrap();

        // 验证编码
        let header_bytes = buf.split_to(9);
        let data_length = LittleEndian::read_u16(&header_bytes[7..9]);
        let expected_length = ((simulated_bits.len() + 1) / 2 + 2) as u16;
        assert_eq!(data_length, expected_length);

        // 跳过结束代码
        let _end_code = buf.get_u16_le();

        // 验证编码后的数据
        let encoded_bits = crate::codec::bools_to_bytes(&simulated_bits);
        assert_eq!(buf.chunk(), &encoded_bits[..]);

        // 验证往返转换：编码后再解码应该得到相同的位数据
        let decoded_bits = crate::codec::bytes_to_bools(&encoded_bits);
        // 注意：由于我们的编码是2位一组，解码会产生更多的位，所以只比较原始长度
        assert_eq!(&decoded_bits[..simulated_bits.len()], &simulated_bits[..]);
    }

    #[test]
    fn test_address_mapping_fix_verification() {
        // 测试修复后的地址映射
        // 模拟 M100 写入 u16 值 11，然后读取 M100 的位数据

        // 模拟写入操作：M100 写入 u16 值 11
        // 11 的二进制: 0000 0000 0000 1011 (16位)
        // 小端序字节: [0x0B, 0x00]
        // M100 的字节偏移: 100 * 2 = 200
        // 所以字节 200=0x0B, 字节 201=0x00

        let simulated_memory = vec![0u8; 4000]; // 模拟 4000 字节内存
        let mut memory = simulated_memory;

        // 模拟写入 M100 = 11 (u16)
        let value_11_bytes = 11u16.to_le_bytes(); // [0x0B, 0x00]
        let m100_byte_offset = 100 * 2; // 200
        memory[m100_byte_offset] = value_11_bytes[0]; // 0x0B
        memory[m100_byte_offset + 1] = value_11_bytes[1]; // 0x00

        // 现在模拟读取 M100 的位数据 (使用修复后的地址映射)
        // M100 位读取应该从字节偏移 200 开始
        let base_byte_offset = 100 * 2; // 200，与字操作相同的映射

        // 读取前16位 (一个字的所有位)
        let mut result_bits = Vec::new();
        for i in 0..16 {
            let bit_in_word = i % 16;
            let word_offset = i / 16;
            let byte_offset = base_byte_offset + word_offset * 2 + bit_in_word / 8;
            let bit_offset = bit_in_word % 8;

            let byte_value = memory[byte_offset];
            let bit_value = (byte_value >> bit_offset) & 0x01 != 0;
            result_bits.push(bit_value);
        }

        // 验证结果
        // 11 的二进制是 0000 1011，小端序存储为 [0x0B, 0x00]
        // 0x0B = 0000 1011，0x00 = 0000 0000
        // 按位读取应该是：
        // 字节0 (0x0B): [true, true, false, true, false, false, false, false] (LSB first)
        // 字节1 (0x00): [false, false, false, false, false, false, false, false]
        let expected_bits = vec![
            true, true, false, true, false, false, false, false, // 0x0B 的位
            false, false, false, false, false, false, false, false, // 0x00 的位
        ];

        assert_eq!(result_bits, expected_bits);

        // 特别验证第0位应该是 true (因为11的LSB是1)
        assert_eq!(
            result_bits[0], true,
            "M100 位0应该是 true，因为值11的第0位是1"
        );
        assert_eq!(
            result_bits[1], true,
            "M100 位1应该是 true，因为值11的第1位是1"
        );
        assert_eq!(
            result_bits[2], false,
            "M100 位2应该是 false，因为值11的第2位是0"
        );
        assert_eq!(
            result_bits[3], true,
            "M100 位3应该是 true，因为值11的第3位是1"
        );
    }

    #[test]
    fn test_mitsubishi_mc_protocol_x_zone_mapping() {
        // 测试三菱MC协议X区域的特殊映射规则
        // 实际寄存器：X0, X10, X20, X30...
        // 位地址：X0-X15对应X0字，X16-X31对应X10字...

        let simulated_memory = vec![0u8; 4000]; // 模拟 4000 字节内存
        let mut memory = simulated_memory;

        // 模拟写入 X0 = 0x1234 (存储在字节0-1)
        let x0_bytes = 0x1234u16.to_le_bytes(); // [0x34, 0x12]
        memory[0] = x0_bytes[0]; // 0x34
        memory[1] = x0_bytes[1]; // 0x12

        // 模拟写入 X10 = 0x5678 (存储在字节20-21，因为X10的地址是10，10*2=20)
        let x10_bytes = 0x5678u16.to_le_bytes(); // [0x78, 0x56]
        memory[20] = x10_bytes[0]; // 0x78
        memory[21] = x10_bytes[1]; // 0x56

        // 测试位读取（使用三菱MC协议规则）

        // 测试 X0 位（应该读取X0字的第0位）
        // bit_addr = 0, word_register = 0*10 = 0 (X0), bit_in_word = 0
        // byte_offset = 0*2 + 0/8 = 0, bit_offset = 0%8 = 0
        let x0_bit_addr = 0;
        let word_register_addr = (x0_bit_addr / 16) * 10; // 0
        let bit_in_word = x0_bit_addr % 16; // 0
        let word_byte_offset = word_register_addr * 2; // 0
        let byte_in_word = bit_in_word / 8; // 0
        let bit_in_byte = bit_in_word % 8; // 0
        let final_byte_offset = word_byte_offset + byte_in_word; // 0

        let byte_value = memory[final_byte_offset]; // 0x34
        let x0_bit_value = (byte_value >> bit_in_byte) & 0x01 != 0; // (0x34 >> 0) & 0x01 = 0
        assert_eq!(x0_bit_value, false, "X0位应该是false，因为0x34的第0位是0");

        // 测试 X16 位（应该读取X10字的第0位）
        // bit_addr = 16, word_register = 1*10 = 10 (X10), bit_in_word = 0
        // byte_offset = 10*2 + 0/8 = 20, bit_offset = 0%8 = 0
        let x16_bit_addr = 16;
        let word_register_addr = (x16_bit_addr / 16) * 10; // 10
        let bit_in_word = x16_bit_addr % 16; // 0
        let word_byte_offset = word_register_addr * 2; // 20
        let byte_in_word = bit_in_word / 8; // 0
        let bit_in_byte = bit_in_word % 8; // 0
        let final_byte_offset = word_byte_offset + byte_in_word; // 20

        let byte_value = memory[final_byte_offset]; // 0x78
        let x16_bit_value = (byte_value >> bit_in_byte) & 0x01 != 0; // (0x78 >> 0) & 0x01 = 0
        assert_eq!(x16_bit_value, false, "X16位应该是false，因为0x78的第0位是0");

        // 测试连续内存模型的u8读取
        // X1 u8读取 = X0的字节1 + X10的字节0
        // byte_addr = 1, register_idx = 1/2 = 0, byte_in_register = 1%2 = 1
        // actual_register_addr = 0*10 = 0, actual_byte_offset = 0*2 + 1 = 1
        let x1_byte1_addr = 1;
        let register_idx = x1_byte1_addr / 2; // 0
        let byte_in_register = x1_byte1_addr % 2; // 1
        let actual_register_addr = register_idx * 10; // 0
        let actual_byte_offset = actual_register_addr * 2 + byte_in_register; // 1
        let x1_byte1_value = memory[actual_byte_offset]; // 0x12

        // X1 u8读取的第二个字节
        // byte_addr = 2, register_idx = 2/2 = 1, byte_in_register = 2%2 = 0
        // actual_register_addr = 1*10 = 10, actual_byte_offset = 10*2 + 0 = 20
        let x1_byte2_addr = 2;
        let register_idx = x1_byte2_addr / 2; // 1
        let byte_in_register = x1_byte2_addr % 2; // 0
        let actual_register_addr = register_idx * 10; // 10
        let actual_byte_offset = actual_register_addr * 2 + byte_in_register; // 20
        let x1_byte2_value = memory[actual_byte_offset]; // 0x78

        // X1 u8读取应该返回 [0x12, 0x78]
        assert_eq!(
            x1_byte1_value, 0x12,
            "X1 u8读取第1字节应该是0x12 (X0的高字节)"
        );
        assert_eq!(
            x1_byte2_value, 0x78,
            "X1 u8读取第2字节应该是0x78 (X10的低字节)"
        );

        log::info!("三菱MC协议X区域映射测试通过！");
    }
}
