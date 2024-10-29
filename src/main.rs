use std::net::SocketAddr;
use tokio::io;
use tokio_mc::{
    client::{tcp::TcpClient, Context, Reader},
    frame::SoftElementCode,
};

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    let addr = "192.168.110.210:5000".parse::<SocketAddr>().unwrap();
    // 等待 TcpClient::new 的 Future 完成，获得 TcpClient 实例
    let tcp_client = TcpClient::new(addr).await?;

    // 将已连接的 TcpClient 实例传递给 Context
    let mut context = Context::new(tcp_client);

    // 调用 read_bits 方法
    let result = context.read_words(0, 10, SoftElementCode::D).await;
    println!("Read bits response: {:?}", result);

    Ok(())
}
