use std::{io, net::SocketAddr, time::Duration};

use crate::client::tcp::TcpClient;

use super::Context;

pub fn connect(socket_addr: SocketAddr) -> io::Result<Context<TcpClient>>  {
    // 创建一个新的 Tokio 运行时
    let runtime = tokio::runtime::Runtime::new()?;
    // 通过运行时创建异步 TcpClient
    let tcp_client = runtime.block_on(TcpClient::new(socket_addr))?;

    // 传递 TcpClient 实例来初始化同步 Context
    let context = Context::new(tcp_client, runtime, Some(Duration::from_secs(1)));

    Ok(context)
}
