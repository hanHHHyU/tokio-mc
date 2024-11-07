use bytes::{BufMut, Bytes, BytesMut};

pub type HeaderByte = Bytes;

pub struct RequestHeader(pub HeaderByte);

impl RequestHeader {
    /// 构造三菱 MC 3E 协议头部
    pub fn new() -> Self {
        // 使用 BytesMut 动态缓冲区
        let mut buf = BytesMut::new();

        // 写入固定的头部
        #[cfg(feature = "3e")]
        buf.put_u16_le(0x0050); // 3E 协议头
        buf.put_u8(0x00); // 网络编号，固定 00
        buf.put_u8(0xFF); // PLC 编号，固定 FF
        buf.put_u16_le(0x03FF); // 目标模块 IO 编号
        buf.put_u8(0x00); // 目标模块站号，固定 00
        buf.put_u16_le(0x000C); // 请求数据的长度（根据实际情况调整）
        buf.put_u16_le(0x0010); // 监视定时器

        // 将 BytesMut 冻结为不可变的 Bytes
        RequestHeader(buf.freeze())
    }

    /// 获取请求头的字节数组
    pub fn bytes(&self) -> &[u8] {
        &self.0
    }

    /// 获取响应头的字节数组长度
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

pub struct ResponseHeader(pub HeaderByte);

impl ResponseHeader {
    pub fn new() -> Self {
        // 使用 BytesMut 动态缓冲区
        let mut buf = BytesMut::new();

        // 写入固定的头部
        buf.put_u8(0xD0); // 3E 协议头
        buf.put_u8(0x00); // 固定 00
        buf.put_u8(0x00); // 网络编号，固定 00
        buf.put_u8(0xFF); // PLC 编号，固定 FF
        buf.put_u16_le(0x03FF); // 目标模块 IO 编号
        buf.put_u8(0x00);
        buf.put_u16_le(2); // 长度默认
                           // buf.put_u16_le(0); // 代码，默认成功 00
        // buf.put_u16_le(2); // 回馈代码
        //                    // D0 00 00 FF FF 03 00

        // 将 BytesMut 冻结为不可变的 Bytes
        ResponseHeader(buf.freeze())
    }

    // /// 获取响应头的字节数组
    // pub fn bytes(&self) -> &[u8] {
    //     &self.0
    // }

    /// 获取响应头的字节数组长度
    pub fn len(&self) -> usize {
        self.0.len()
    }
}
