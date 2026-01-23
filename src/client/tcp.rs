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
        .map_err(|_| {
            Error::Transport(io::Error::new(
                io::ErrorKind::TimedOut,
                "Connection timeout",
            ))
        })?
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

#[cfg(test)]
mod tcp_tests {
    use super::*;
    use crate::client::Reader;
    use crate::codec::bools_to_bytes;
    use crate::header::ResponseHeader;
    use crate::test_common::create_test_listener;
    use byteorder::{ByteOrder, LittleEndian};
    use bytes::BytesMut;
    use std::borrow::Cow;
    use std::io;
    use tokio::io::{duplex, AsyncReadExt, AsyncWriteExt};
    use tokio::time::Duration;

    fn encode_response_bytes(response: Response) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(&[0x00, 0x00]);
        match response {
            Response::ReadU8s(values) => payload.extend_from_slice(&values),
            Response::ReadBits(values) => {
                let bytes = bools_to_bytes(&values);
                payload.extend_from_slice(&bytes);
            }
            Response::WriteU8s() | Response::WriteBits() => {}
        }

        let header = ResponseHeader::new();
        let mut header_bytes = BytesMut::from(&header.0[..]);
        let header_len = header_bytes.len();
        LittleEndian::write_u16(
            &mut header_bytes[header_len - 2..header_len],
            payload.len() as u16,
        );

        let mut frame = Vec::with_capacity(header_len + payload.len());
        frame.extend_from_slice(&header_bytes);
        frame.extend_from_slice(&payload);
        frame
    }

    #[tokio::test]
    async fn test_tcp_connect_success() {
        let (listener, addr) = create_test_listener().await;

        let accept_task = tokio::spawn(async move { listener.accept().await });

        let result = connect(addr).await;

        assert!(result.is_ok());
        let _ = accept_task.await;
    }

    #[tokio::test]
    async fn test_tcp_connect_with_timeout_success() {
        let (listener, addr) = create_test_listener().await;

        let accept_task = tokio::spawn(async move { listener.accept().await });

        let result = connect_with_timeout(addr, Duration::from_secs(1)).await;

        assert!(result.is_ok());
        let _ = accept_task.await;
    }

    #[tokio::test]
    async fn test_tcp_connect_with_timeout_expires() {
        let addr = "192.0.2.1:5000".parse().unwrap();

        let result = connect_with_timeout(addr, Duration::from_millis(100)).await;

        match result {
            Err(Error::Transport(err)) => assert_eq!(err.kind(), io::ErrorKind::TimedOut),
            other => panic!("expected timeout error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_tcp_connect_to_invalid_address_fails() {
        let addr = "0.0.0.0:0".parse().unwrap();

        let result = connect(addr).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_tcp_client_call_read() {
        let (client_io, mut server_io) = duplex(1024);
        let mut client = TcpClient::new(client_io);

        let server_task = tokio::spawn(async move {
            let mut buf = vec![0u8; 256];
            let _ = server_io.read(&mut buf).await.unwrap();
            let response = encode_response_bytes(Response::ReadU8s(vec![0x10, 0x20]));
            server_io.write_all(&response).await.unwrap();
        });

        let response = client
            .call(Request::ReadU8s("D100".into(), 2))
            .await
            .unwrap();

        assert_eq!(response, Response::ReadU8s(vec![0x10, 0x20]));
        let _ = server_task.await;
    }

    #[tokio::test]
    async fn test_tcp_client_call_write() {
        let (client_io, mut server_io) = duplex(1024);
        let mut client = TcpClient::new(client_io);

        let server_task = tokio::spawn(async move {
            let mut buf = vec![0u8; 256];
            let _ = server_io.read(&mut buf).await.unwrap();
            let response = encode_response_bytes(Response::WriteU8s());
            server_io.write_all(&response).await.unwrap();
        });

        let request = Request::WriteU8s("D100".into(), Cow::Borrowed(&[0x01, 0x02]));
        let response = client.call(request).await.unwrap();

        assert_eq!(response, Response::WriteU8s());
        let _ = server_task.await;
    }

    #[tokio::test]
    async fn test_tcp_client_disconnect() {
        let (client_io, _server_io) = duplex(64);
        let mut client = TcpClient::new(client_io);

        let result = client.disconnect().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_tcp_client_call_after_disconnect_fails() {
        let (client_io, _server_io) = duplex(64);
        let mut client = TcpClient::new(client_io);

        client.disconnect().await.unwrap();
        let result = client.call(Request::ReadU8s("D100".into(), 1)).await;

        match result {
            Err(Error::Transport(err)) => assert_eq!(err.kind(), io::ErrorKind::NotConnected),
            other => panic!("expected NotConnected error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_tcp_client_attach_custom_transport() {
        let (client_io, mut server_io) = duplex(1024);
        let mut ctx = attach(client_io);

        let server_task = tokio::spawn(async move {
            let mut buf = vec![0u8; 256];
            let _ = server_io.read(&mut buf).await.unwrap();
            let response = encode_response_bytes(Response::ReadU8s(vec![0xAA, 0xBB]));
            server_io.write_all(&response).await.unwrap();
        });

        let result = ctx.read_u8s("D100", 2).await;

        assert_eq!(result.unwrap(), vec![0xAA, 0xBB]);
        let _ = server_task.await;
    }
}
