use std::{io, net::SocketAddr};

use async_trait::async_trait;
use bytes::Bytes;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::error;

use super::{Client, Context, ExceptionCode, Request, Response};

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
    async fn call(
        &mut self,
        request: Request<'_>,
    ) -> Result<Result<Response, ExceptionCode>, error::Error> {
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
        let response_bytes = Bytes::copy_from_slice(&buffer[..n]);
        let response = Response::try_from((response_bytes, request)).unwrap();

        Ok(Ok(response))
    }
}
