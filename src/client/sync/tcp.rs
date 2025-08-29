use std::{io, net::SocketAddr, time::Duration};
use tokio::net::TcpStream;

use crate::client::tcp::TcpClient;

use super::Context;
use crate::Error;

pub fn connect(socket_addr: SocketAddr) -> Result<Context<TcpClient>, Error> {
    // Create a new Tokio runtime
    let runtime = tokio::runtime::Runtime::new()?;

    // Connect to TCP server through runtime and create TcpClient
    let tcp_client = runtime.block_on(async {
        let stream = TcpStream::connect(socket_addr).await?;
        Ok::<TcpClient, Error>(TcpClient::new(stream))
    })?;

    // Pass TcpClient instance to initialize sync Context
    let context = Context::new(tcp_client, runtime, Some(Duration::from_secs(1)));

    Ok(context)
}

/// Connect with custom timeouts
pub fn connect_with_timeout(
    socket_addr: SocketAddr,
    connect_timeout: Duration,
    operation_timeout: Option<Duration>,
) -> Result<Context<TcpClient>, Error> {
    // Create a new Tokio runtime
    let runtime = tokio::runtime::Runtime::new()?;

    // Connect to TCP server through runtime and create TcpClient with connection timeout
    let tcp_client = runtime.block_on(async {
        let stream = tokio::time::timeout(connect_timeout, TcpStream::connect(socket_addr))
            .await
            .map_err(|_| Error::Transport(io::Error::new(io::ErrorKind::TimedOut, "Connection timeout")))?
            .map_err(Error::Transport)?;
        Ok::<TcpClient, Error>(TcpClient::new(stream))
    })?;

    // Pass TcpClient instance to initialize sync Context
    let context = Context::new(tcp_client, runtime, operation_timeout);

    Ok(context)
}
