use std::{fmt, io, net::SocketAddr, time::Duration};

use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
};
use tokio_util::codec::Framed;

use crate::{codec::tcp::McClientCodec, Error};

use super::{Client, Context, Request, Response};

/// Establish a direct connection to a MC TCP device
pub async fn connect(socket_addr: SocketAddr) -> Result<Context<TcpClient>, Error> {
    let transport = TcpStream::connect(socket_addr).await?;
    let client = TcpClient::new(transport);
    let context = Context::<TcpClient>::new(client);
    Ok(context)
}

/// Establish a direct connection to a MC TCP device with timeout
pub async fn connect_with_timeout(
    socket_addr: SocketAddr,
    timeout: Duration,
) -> Result<Context<TcpClient>, Error> {
    let transport = tokio::time::timeout(timeout, TcpStream::connect(socket_addr))
        .await
        .map_err(|_| Error::Transport(io::Error::new(io::ErrorKind::TimedOut, "Connection timeout")))?
        .map_err(Error::Transport)?;
    
    let client = TcpClient::new(transport);
    let context = Context::<TcpClient>::new(client);
    Ok(context)
}

/// Attach a new client context to a transport connection
pub fn attach<T>(transport: T) -> Context<TcpClient<T>>
where
    T: AsyncRead + AsyncWrite + Send + Unpin + fmt::Debug + 'static,
{
    let client = TcpClient::new(transport);
    Context::<TcpClient<T>>::new(client)
}

#[derive(Debug)]
pub struct TcpClient<T = TcpStream> {
    framed: Option<Framed<T, McClientCodec>>,
}

impl<T> TcpClient<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    /// Create a new TcpClient with the given transport
    pub fn new(transport: T) -> Self {
        let framed = Framed::new(transport, McClientCodec::new());
        Self {
            framed: Some(framed),
        }
    }

    fn framed(&mut self) -> io::Result<&mut Framed<T, McClientCodec>> {
        let Some(framed) = &mut self.framed else {
            return Err(io::Error::new(io::ErrorKind::NotConnected, "disconnected"));
        };
        Ok(framed)
    }

    async fn disconnect(&mut self) -> io::Result<()> {
        if let Some(framed) = self.framed.take() {
            // Proper cleanup of the connection
            let transport = framed.into_inner();
            drop(transport);
        }
        Ok(())
    }
}

#[async_trait]
impl<T> Client for TcpClient<T>
where
    T: fmt::Debug + AsyncRead + AsyncWrite + Send + Unpin,
{
    async fn call(&mut self, request: Request<'_>) -> Result<Response, Error> {
        let framed = self.framed()?;

        // Clear any existing data in the read buffer
        framed.read_buffer_mut().clear();

        // Send the request
        framed.send(request.clone()).await?;

        // Receive the raw response bytes
        let raw_response = framed
            .next()
            .await
            .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "Connection closed"))??;

        // Convert raw bytes to Vec<Bytes> and use ClientDecoder for parsing
        let bytes_vec = vec![raw_response];
        let response = crate::codec::ClientDecoder::decode(bytes_vec, request)?;

        Ok(response)
    }

    async fn disconnect(&mut self) -> io::Result<()> {
        self.disconnect().await
    }
}
