// use std::{net::SocketAddr, time::Duration};
// use tokio::io;
// use tokio_mc::{
//     client::{tcp::TcpClient, Context, Reader}
// };

// #[tokio::main]
// async fn main() -> Result<(), io::Error> {
//     let addr = "192.168.110.210:5000".parse::<SocketAddr>().unwrap();
//     // 等待 TcpClient::new 的 Future 完成，获得 TcpClient 实例
//     let tcp_client = TcpClient::new(addr).await?;

//     // 将已连接的 TcpClient 实例传递给 Context

//     let mut context: Context<TcpClient> = Context::<TcpClient>::new(tcp_client);
//     // 调用 read_bits 方法
//     let result = context.read_words("D0", 10).await;
//     println!("Read bits response: {:?}", result);

//     Ok(())
// }

// // fn main() -> Result<(), Box<dyn std::error::Error>> {
// //     // 设置目标服务器地址
// //     let addr = "192.168.110.210:5000".parse::<SocketAddr>()?;

// //     // 创建一个新的 Tokio 运行时
// //     let runtime = tokio::runtime::Runtime::new()?;

// //     // 通过运行时创建异步 TcpClient
// //     let tcp_client = runtime.block_on(TcpClient::new(addr))?;

// //     // 传递 TcpClient 实例来初始化同步 Context
// //     let mut context =
// //         tokio_mc::client::sync::Context::new(tcp_client, runtime, Some(Duration::from_secs(1)));

// //     // 读取 10 个字的示例
// //     let read_result = context.read_words(0, 10, SoftElementCode::D)?;
// //     println!("Read words response: {:?}", read_result);

// //     // // 调用其他同步方法
// //     // let response = context.call(your_request_here)?; // 替换为实际请求
// //     // println!("Call response: {:?}", response);

// //     Ok(())
// // }

use once_cell::sync::Lazy;
use std::io;
use std::net::SocketAddr;
use tokio::sync::{Mutex, OnceCell};
use tokio_mc::client::tcp::TcpClient;
use tokio_mc::client::{Context, Reader};

// 假设 TcpClient 和 Context 已经定义
static CONTEXT: Lazy<OnceCell<Mutex<Context<TcpClient>>>> = Lazy::new(OnceCell::new);

async fn initialize_context() -> io::Result<()> {
    let addr = "192.168.110.210:5000".parse::<SocketAddr>().unwrap();
    let tcp_client = TcpClient::new(addr).await?;
    let context = Context::new(tcp_client);

    // 初始化 CONTEXT 变量，仅第一次调用会进行初始化
    CONTEXT.set(Mutex::new(context)).unwrap();
    Ok(())
}

async fn get_context() -> &'static Mutex<Context<TcpClient>> {
    CONTEXT.get().expect("Context has not been initialized")
}

#[tokio::main]
async fn main() -> io::Result<()> {
    // 初始化 Context
    initialize_context().await?;

    // 从全局 CONTEXT 获取实例并调用 `read_words`
    let context = get_context().await;
    let mut context = context.lock().await;
    let result = context.read_words("D0", 10).await;

    println!("Read bits response: {:?}", result);
    Ok(())
}
