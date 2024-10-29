use std::{net::SocketAddr, time::Duration};
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

    let mut context: Context<TcpClient> = Context::<TcpClient>::new(tcp_client);
    // 调用 read_bits 方法
    let result = context.read_words(0, 10, SoftElementCode::D).await;
    println!("Read bits response: {:?}", result);

    Ok(())
}

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     // 设置目标服务器地址
//     let addr = "192.168.110.210:5000".parse::<SocketAddr>()?;

//     // 创建一个新的 Tokio 运行时
//     let runtime = tokio::runtime::Runtime::new()?;

//     // 通过运行时创建异步 TcpClient
//     let tcp_client = runtime.block_on(TcpClient::new(addr))?;

//     // 传递 TcpClient 实例来初始化同步 Context
//     let mut context =
//         tokio_mc::client::sync::Context::new(tcp_client, runtime, Some(Duration::from_secs(1)));

//     // 读取 10 个字的示例
//     let read_result = context.read_words(0, 10, SoftElementCode::D)?;
//     println!("Read words response: {:?}", read_result);

//     // // 调用其他同步方法
//     // let response = context.call(your_request_here)?; // 替换为实际请求
//     // println!("Call response: {:?}", response);

//     Ok(())
// }
