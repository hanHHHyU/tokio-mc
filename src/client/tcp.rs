use std::{io, net::SocketAddr};

use async_trait::async_trait;
use byteorder::ByteOrder;
use byteorder::LittleEndian;
use bytes::Bytes;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::{header::ResponseHeader, Error};

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
    // stream: TcpStream, // 直接保存 TcpStream 实例
    stream: TcpStream,
}

impl TcpClient {
    /// 创建 TcpClient 实例并建立连接
    pub async fn new(addr: SocketAddr) -> Result<Self, io::Error> {
        let stream = TcpStream::connect(addr).await?;
        Ok(Self { stream })
    }
}

#[async_trait]
impl Client for TcpClient {
    async fn call(&mut self, request: Request<'_>) -> Result<Response, Error> {
        let request_parts: Vec<Bytes> = Vec::try_from(request.clone()).unwrap();

        let mut complete_response = Vec::new(); // 用于收集所有响应数据

        // println!("request_parts {:?}", request_parts.len());

        let header_len = ResponseHeader::new().len();

        for part in request_parts {
            // println!("Read {:?}", part);

            // 1. 逐个发送 Vec<Bytes> 中的每个片段
            self.stream.write_all(&part[..]).await?;

            let mut header = vec![0; header_len];

            self.stream.read_exact(&mut header).await?;

            // println!("读取帧{:?}", header);

            let frame_length = LittleEndian::read_u16(&header[header_len - 2..header_len]) as usize;
            // 根据解析出的长度读取剩余帧
            let mut buffer = vec![0; frame_length];
            // println!("读取帧长度{}", frame_length);
            self.stream.read_exact(&mut buffer).await?;

            let response_bytes: Bytes = Bytes::copy_from_slice(&buffer);

            complete_response.push(response_bytes);
        }

        let response = Response::try_from((complete_response, request))?;

        Ok(response)
    }
}
