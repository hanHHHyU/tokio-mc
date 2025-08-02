use std::{future::Future, io, net::SocketAddr};

use async_trait::async_trait;
use futures_util::{FutureExt as _, SinkExt as _, StreamExt as _};
use socket2::{Domain, Socket, Type};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::{TcpListener, TcpStream},
};
use tokio_util::codec::Framed;

use crate::{
    codec::tcp::ServerCodec,
    frame::{Request, Response},
};

use super::Service;

#[async_trait]
pub trait BindSocket {
    type Error;

    async fn bind_socket(addr: SocketAddr) -> Result<Socket, Self::Error>;
}

/// Server termination status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Terminated {
    /// Server finished normally
    Finished,
    /// Server was aborted by signal
    Aborted,
}

/// Accept unencrypted TCP connections.
pub fn accept_tcp_connection<S, NewService>(
    stream: TcpStream,
    socket_addr: SocketAddr,
    new_service: NewService,
) -> io::Result<Option<(S, TcpStream)>>
where
    S: Service<Request = Request<'static>, Response = Response> + Send + Sync + 'static,
    S::Exception: Send,
    NewService: Fn(SocketAddr) -> io::Result<Option<S>>,
{
    let service = new_service(socket_addr)?;
    Ok(service.map(|service| (service, stream)))
}

#[derive(Debug)]
pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub fn new(listener: TcpListener) -> Self {
        Self { listener }
    }

    /// Listens for incoming connections and starts a MC TCP server task for
    /// each connection.
    ///
    /// `OnConnected` is responsible for creating both the service and the
    /// transport layer for the underlying TCP stream. If `OnConnected` returns
    /// with `Err` then listening stops and [`Self::serve()`] returns with an error.
    /// If `OnConnected` returns `Ok(None)` then the connection is rejected
    /// but [`Self::serve()`] continues listening for new connections.
    pub async fn serve<S, T, F, OnConnected, OnProcessError>(
        &self,
        on_connected: &OnConnected,
        on_process_error: OnProcessError,
    ) -> io::Result<()>
    where
        S: Service<Request = Request<'static>, Response = Response> + Send + Sync + 'static,
        S::Exception: Send + std::fmt::Debug,
        T: AsyncRead + AsyncWrite + Unpin + Send + 'static,
        OnConnected: Fn(TcpStream, SocketAddr) -> F,
        F: Future<Output = io::Result<Option<(S, T)>>>,
        OnProcessError: FnOnce(io::Error) + Clone + Send + 'static,
    {
        loop {
            let (stream, socket_addr) = self.listener.accept().await?;
            log::debug!("Accepted connection from {socket_addr}");

            let Some((service, transport)) = on_connected(stream, socket_addr).await? else {
                log::debug!("No service for connection from {socket_addr}");
                continue;
            };
            let on_process_error = on_process_error.clone();

            let framed = Framed::new(transport, ServerCodec::default());

            tokio::spawn(async move {
                log::debug!("Processing requests from {socket_addr}");
                if let Err(err) = process(framed, service).await {
                    on_process_error(err);
                }
            });
        }
    }

    /// Start an abortable MC TCP server task.
    ///
    /// Warning: Request processing is not scoped and could be aborted at any internal await point!
    /// See also: <https://rust-lang.github.io/wg-async/vision/roadmap/scopes.html#cancellation>
    pub async fn serve_until<S, T, F, X, OnConnected, OnProcessError>(
        self,
        on_connected: &OnConnected,
        on_process_error: OnProcessError,
        abort_signal: X,
    ) -> io::Result<Terminated>
    where
        S: Service<Request = Request<'static>, Response = Response> + Send + Sync + 'static,
        S::Exception: Send + std::fmt::Debug,
        T: AsyncRead + AsyncWrite + Unpin + Send + 'static,
        X: Future<Output = ()> + Sync + Send + Unpin + 'static,
        OnConnected: Fn(TcpStream, SocketAddr) -> F,
        F: Future<Output = io::Result<Option<(S, T)>>>,
        OnProcessError: FnOnce(io::Error) + Clone + Send + 'static,
    {
        let abort_signal = abort_signal.fuse();
        tokio::select! {
            res = self.serve(on_connected, on_process_error) => {
                res.map(|()| Terminated::Finished)
            },
            () = abort_signal => {
                Ok(Terminated::Aborted)
            }
        }
    }
}

/// The request-response loop spawned by [`Server::serve`] for each client
async fn process<S, T>(mut framed: Framed<T, ServerCodec>, service: S) -> io::Result<()>
where
    S: Service<Request = Request<'static>, Response = Response> + Send + Sync + 'static,
    S::Exception: Send + std::fmt::Debug,
    T: AsyncRead + AsyncWrite + Unpin,
{
    loop {
        let Some(request_bytes) = framed.next().await.transpose().inspect_err(|err| {
            log::debug!("Failed to receive and decode request: {err}");
        })?
        else {
            log::debug!("TCP socket has been closed");
            break;
        };

        log::debug!("Received request: {:02X?}", request_bytes);

        let req = crate::codec::ServerDecoder::decode(request_bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Parse error: {e}")))?;

        let fc = req.function_code();
        let result: Result<Response, <S as Service>::Exception> = service.call(req).await;

        match result {
            Ok(resp) => {
                framed.send(resp).await.inspect_err(|err| {
                    log::debug!("Failed to send response (function = {fc}): {err}");
                })?;
            }
            Err(exc) => {
                log::warn!("Service error for function {fc}: {exc:?}");
                // For error cases, send an appropriate error response
                // This could be enhanced to return proper error codes based on the exception type
                let error_response = Response::WriteU8s();
                framed.send(error_response).await.inspect_err(|err| {
                    log::debug!("Failed to send error response (function = {fc}): {err}");
                })?;
            }
        }
    }

    Ok(())
}

/// Start TCP listener - configure and open TCP socket
#[allow(unused)]
fn listener(addr: SocketAddr, workers: usize) -> io::Result<TcpListener> {
    let listener = match addr {
        SocketAddr::V4(_) => Socket::new(Domain::IPV4, Type::STREAM, None)?,
        SocketAddr::V6(_) => Socket::new(Domain::IPV6, Type::STREAM, None)?,
    };
    configure_tcp(workers, &listener)?;
    listener.reuse_address()?;
    listener.bind(&addr.into())?;
    listener.listen(1024)?;
    TcpListener::from_std(listener.into())
}

#[cfg(unix)]
#[allow(unused)]
fn configure_tcp(workers: usize, tcp: &Socket) -> io::Result<()> {
    if workers > 1 {
        tcp.reuse_port()?;
    }
    Ok(())
}

#[cfg(windows)]
#[allow(unused)]
fn configure_tcp(_workers: usize, _tcp: &Socket) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{future, sync::Arc, time::Duration};
    use tokio::io::{duplex, AsyncReadExt, AsyncWriteExt};
    use tokio_util::codec::Framed;

    use crate::server::service::Service;

    #[derive(Clone)]
    struct DummyService {
        response: Response,
    }

    impl Service for DummyService {
        type Request = Request<'static>;
        type Response = Response;
        type Exception = std::io::Error;
        type Future = future::Ready<Result<Self::Response, Self::Exception>>;

        fn call(&self, _req: Self::Request) -> Self::Future {
            future::ready(Ok(self.response.clone()))
        }
    }

    #[derive(Clone)]
    struct EchoService;

    impl Service for EchoService {
        type Request = Request<'static>;
        type Response = Response;
        type Exception = std::io::Error;
        type Future = future::Ready<Result<Self::Response, Self::Exception>>;

        fn call(&self, req: Self::Request) -> Self::Future {
            let response = match req {
                Request::ReadU8s(_, qty) => {
                    // 模拟读取操作，返回指定数量的测试数据
                    let test_data = (0..qty).map(|i| (i % 256) as u8).collect();
                    Response::ReadU8s(test_data)
                }
                Request::WriteU8s(_, data) => {
                    // 模拟写操作成功
                    log::debug!("Writing {} bytes", data.len());
                    Response::WriteU8s()
                }
            };
            future::ready(Ok(response))
        }
    }

    #[derive(Clone)]
    struct ErrorService;

    impl Service for ErrorService {
        type Request = Request<'static>;
        type Response = Response;
        type Exception = std::io::Error;
        type Future = future::Ready<Result<Self::Response, Self::Exception>>;

        fn call(&self, _req: Self::Request) -> Self::Future {
            future::ready(Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Service error for testing",
            )))
        }
    }

    #[tokio::test]
    async fn test_process_reads_instruction_code_and_exits_on_eof() {
        let (mut client, server) = duplex(1024);
        let framed = Framed::new(server, ServerCodec::default());

        // 使用正确的MC协议格式
        let bytes = [
            0xD0, 0x00, 0x00, 0xFF, 0xFF, 0x03, 0x00, 0x0E, 0x00, // 头部
            0x10, 0x00, 0x01, 0x14, 0x00, 0x00, // 写U8s命令
            0x00, 0x00, 0x00, // D0地址
            0xA8, 0x02, 0x00, // 数量2
            0x58, 0x1B, // 要写入的数据
        ];

        client.write_all(&bytes).await.unwrap();
        client.shutdown().await.unwrap();

        let svc = DummyService {
            response: Response::ReadU8s(vec![42, 43]),
        };
        let result = process(framed, svc).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_multiple_read_operations() {
        let (mut client, server) = duplex(1024);
        let framed = Framed::new(server, ServerCodec::default());

        // 第一个读请求
        let read_request1 = [
            0xD0, 0x00, 0x00, 0xFF, 0xFF, 0x03, 0x00, 0x0C, 0x00, // 头部
            0x10, 0x00, 0x01, 0x04, 0x00, 0x00, // 读U8s命令
            0x44, 0x30, 0x00, 0x00, // D0地址
            0x02, 0x00, // 数量2
        ];

        // 第二个读请求
        let read_request2 = [
            0xD0, 0x00, 0x00, 0xFF, 0xFF, 0x03, 0x00, 0x0C, 0x00, // 头部
            0x10, 0x00, 0x01, 0x04, 0x00, 0x00, // 读U8s命令
            0x44, 0x31, 0x00, 0x00, // D1地址
            0x04, 0x00, // 数量4
        ];

        let service = EchoService;

        // 启动处理任务
        let process_task = tokio::spawn(async move { process(framed, service).await });

        // 发送第一个请求
        client.write_all(&read_request1).await.unwrap();

        // 读取第一个响应
        let mut response_buf = vec![0u8; 64];
        let n = client.read(&mut response_buf).await.unwrap();
        assert!(n > 0, "Should receive response for first request");

        // 发送第二个请求
        client.write_all(&read_request2).await.unwrap();

        // 读取第二个响应
        let mut response_buf2 = vec![0u8; 64];
        let n2 = client.read(&mut response_buf2).await.unwrap();
        assert!(n2 > 0, "Should receive response for second request");

        // 关闭连接
        client.shutdown().await.unwrap();

        // 等待处理任务完成
        let result = process_task.await.unwrap();
        assert!(result.is_ok(), "Process should complete successfully");
    }

    #[tokio::test]
    async fn test_write_operation() {
        let (mut client, server) = duplex(1024);
        let framed = Framed::new(server, ServerCodec::default());

        // 写请求数据
        let write_request = [
            0xD0, 0x00, 0x00, 0xFF, 0xFF, 0x03, 0x00, 0x0E, 0x00, // 头部
            0x10, 0x00, 0x01, 0x14, 0x00, 0x00, // 写U8s命令
            0x44, 0x30, 0x00, 0x00, // D0地址
            0x02, 0x00, // 数量2
            0xAA, 0xBB, // 要写入的数据
        ];

        let service = EchoService;

        // 启动处理任务
        let process_task = tokio::spawn(async move { process(framed, service).await });

        // 发送写请求
        client.write_all(&write_request).await.unwrap();

        // 读取响应
        let mut response_buf = vec![0u8; 32];
        let n = client.read(&mut response_buf).await.unwrap();
        assert!(n > 0, "Should receive response for write request");

        // 关闭连接
        client.shutdown().await.unwrap();

        // 等待处理任务完成
        let result = process_task.await.unwrap();
        assert!(result.is_ok(), "Write process should complete successfully");
    }

    #[tokio::test]
    async fn test_mixed_read_write_operations() {
        let (mut client, server) = duplex(1024);
        let framed = Framed::new(server, ServerCodec::default());

        let service = EchoService;

        // 启动处理任务
        let process_task = tokio::spawn(async move { process(framed, service).await });

        // 1. 先发送写请求
        let write_request = [
            0xD0, 0x00, 0x00, 0xFF, 0xFF, 0x03, 0x00, 0x0E, 0x00, 0x10, 0x00, 0x01, 0x14, 0x00,
            0x00, 0x44, 0x30, 0x00, 0x00, 0x03, 0x00, 0x11, 0x22, 0x33,
        ];

        client.write_all(&write_request).await.unwrap();

        // 读取写响应
        let mut buf = vec![0u8; 32];
        let n = client.read(&mut buf).await.unwrap();
        assert!(n > 0, "Should receive write response");

        // 2. 然后发送读请求
        let read_request = [
            0xD0, 0x00, 0x00, 0xFF, 0xFF, 0x03, 0x00, 0x0C, 0x00, 0x10, 0x00, 0x01, 0x04, 0x00,
            0x00, 0x44, 0x30, 0x00, 0x00, 0x03, 0x00,
        ];

        client.write_all(&read_request).await.unwrap();

        // 读取读响应
        let mut buf2 = vec![0u8; 32];
        let n2 = client.read(&mut buf2).await.unwrap();
        assert!(n2 > 0, "Should receive read response");

        // 关闭连接
        client.shutdown().await.unwrap();

        let result = process_task.await.unwrap();
        assert!(
            result.is_ok(),
            "Mixed operations should complete successfully"
        );
    }

    #[tokio::test]
    async fn test_service_error_handling() {
        let (mut client, server) = duplex(1024);
        let framed = Framed::new(server, ServerCodec::default());

        let service = ErrorService;

        let process_task = tokio::spawn(async move { process(framed, service).await });

        // 发送一个请求，服务会返回错误
        let request = [
            0xD0, 0x00, 0x00, 0xFF, 0xFF, 0x03, 0x00, 0x0C, 0x00, 0x10, 0x00, 0x01, 0x04, 0x00,
            0x00, 0x44, 0x30, 0x00, 0x00, 0x01, 0x00,
        ];

        client.write_all(&request).await.unwrap();

        // 应该收到错误响应
        let mut buf = vec![0u8; 32];
        let n = client.read(&mut buf).await.unwrap();
        assert!(n > 0, "Should receive error response");

        client.shutdown().await.unwrap();

        let result = process_task.await.unwrap();
        assert!(
            result.is_ok(),
            "Error handling should not crash the process"
        );
    }

    #[tokio::test]
    async fn test_tcp_server_integration() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let service = Arc::new(EchoService);
        let on_connected = {
            let service = Arc::clone(&service);
            move |stream, socket_addr| {
                let service = Arc::clone(&service);
                async move {
                    accept_tcp_connection(stream, socket_addr, move |_| {
                        Ok(Some(Arc::clone(&service)))
                    })
                }
            }
        };

        let server = Server::new(listener);

        // 启动服务器
        let server_task = tokio::spawn(async move {
            tokio::time::timeout(
                Duration::from_secs(2),
                server.serve(&on_connected, |_err| {}),
            )
            .await
        });

        // 等待服务器启动
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 连接客户端
        let mut stream = TcpStream::connect(addr).await.unwrap();

        // 发送读请求
        let read_request = [
            0xD0, 0x00, 0x00, 0xFF, 0xFF, 0x03, 0x00, 0x0C, 0x00, 0x10, 0x00, 0x01, 0x04, 0x00,
            0x00, 0x44, 0x30, 0x00, 0x00, 0x05, 0x00, // 请求5个字节
        ];

        stream.write_all(&read_request).await.unwrap();

        // 读取响应
        let mut response = vec![0u8; 64];
        let n = stream.read(&mut response).await.unwrap();
        assert!(n > 0, "Should receive response from server");

        // 关闭连接会触发服务器任务退出
        drop(stream);

        // 等待服务器超时退出
        let _result = server_task.await;
    }

    #[tokio::test]
    async fn test_invalid_request_data() {
        let (mut client, server) = duplex(1024);
        let framed = Framed::new(server, ServerCodec::default());

        let service = EchoService;

        let process_task = tokio::spawn(async move { process(framed, service).await });

        // 发送无效的请求数据（头部正确但payload无效）
        let invalid_request = [
            0xD0, 0x00, 0x00, 0xFF, 0xFF, 0x03, 0x00, 0x04, 0x00, // 头部
            0x99, 0x99, 0xFF, 0xFF, // 无效的命令数据
        ];

        client.write_all(&invalid_request).await.unwrap();
        client.shutdown().await.unwrap();

        let result = process_task.await.unwrap();
        // 应该因为解析错误而失败
        assert!(result.is_err(), "Invalid request should cause error");
    }

    #[tokio::test]
    async fn delegate_service_through_deref_for_server() {
        let service = Arc::new(DummyService {
            response: Response::ReadU8s(vec![0x33]),
        });
        let svc = |_socket_addr| Ok(Some(Arc::clone(&service)));
        let on_connected =
            |stream, socket_addr| async move { accept_tcp_connection(stream, socket_addr, svc) };

        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let listener = TcpListener::bind(addr).await.unwrap();
        let server = Server::new(listener);

        // passes type-check is the goal here
        std::mem::drop(server.serve(&on_connected, |_err| {}));
    }
}
