use std::{io, net::SocketAddr};

use async_trait::async_trait;
use byteorder::{ByteOrder, LittleEndian};
use bytes::Bytes;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::{
    frame::map_error_code,
    header::ResponseHeader,
    Error,
};

use super::{Client, Context, Request, Response};

pub async fn connect(socket_addr: SocketAddr) -> io::Result<Context<TcpClient>> {
    // 等待 TcpClient::new 的 Future 完成，获得 TcpClient 实例
    let tcp_client = TcpClient::new(socket_addr).await?;
    let context: Context<TcpClient> = Context::<TcpClient>::new(tcp_client);
    // 返回 `Ok(context)`，这是标准的 `Result` 类型
    Ok(context)
}

#[derive(Debug)]
pub struct TcpClient {
    stream: TcpStream, // 直接保存 TcpStream 实例
}

impl TcpClient {
    /// 创建 TcpClient 实例并建立连接
    pub async fn new(addr: SocketAddr) -> Result<Self, io::Error> {
        let stream = TcpStream::connect(addr).await?;
        println!("Connected to {:?}", addr);
        Ok(Self { stream })
    }
}

#[async_trait]
impl Client for TcpClient {
    async fn call(&mut self, request: Request<'_>) -> Result<Response, Error> {
        println!("Processing request: {:?}", request);

        // 1. 将 Request 转换为 Bytes
        let request_bytes = Bytes::try_from(request.clone()).unwrap();

        // 2. 发送请求
        self.stream.write_all(&request_bytes[..]).await?;

        // 3. 接收响应
        let mut buffer = vec![0; 4096];
        let n = self.stream.read(&mut buffer).await?;

        // 打印接收到的数据（十六进制格式）
        println!("接收到的数据 (十六进制):");
        for byte in &buffer[..n] {
            print!("{:02X} ", byte);
        }
        println!();

        // 4. 解析响应数据，将字节缓冲区转换为 Response
        let response_bytes: Bytes = Bytes::copy_from_slice(&buffer[..n]);

        check_response(&response_bytes)?;
        let response = Response::try_from((response_bytes, request)).unwrap();

        Ok(response)
    }
}

fn check_response(response_bytes: &[u8]) -> Result<(), Error> {
    let header_len = ResponseHeader::new().len();
    // 获取响应字节缓冲区的前 `header_len` 字节，并提取最后两个字节
    let last_two_bytes = &response_bytes[..header_len][header_len - 2..];
    println!(
        "Last two bytes in hex: {:02X} {:02X}",
        last_two_bytes[0], last_two_bytes[1]
    );

    // 将最后两个字节转换为小端格式的 16 位整数
    let last_two = LittleEndian::read_u16(last_two_bytes);

    if let Some(error) = map_error_code(last_two) {
        return Err(error.into());
    }

    Ok(())
}
