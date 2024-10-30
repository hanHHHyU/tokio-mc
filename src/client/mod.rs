// #[cfg(feature = "sync")]
pub mod sync;
pub mod tcp;

use async_trait::async_trait;
use std::{borrow::Cow, fmt::Debug};

use crate::frame::*;
use crate::Error;

#[async_trait]
pub trait Client: Send + Debug {
    /// Invokes a _Modbus_ function.
    async fn call(&mut self, request: Request<'_>) -> Result<Response, Error>;
}

#[async_trait]
pub trait Reader: Client {
    async fn read_bits<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<Bit>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    async fn read_words<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<Word>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;
}

#[async_trait]
pub trait Writer: Client {
    async fn write_multiple_bits<A>(&mut self, addr: &A, bits: &[Bit]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    async fn write_multiple_word<A>(&mut self, addr: &A, words: &[Word]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;
}

/// Asynchronous Modbus client context with generic transport
#[derive(Debug)]
pub struct Context<T: Client> {
    client: T,
}

impl<T: Client> Context<T> {
    pub fn new(client: T) -> Self {
        Self { client }
    }
}

#[async_trait]
impl<T: Client> Client for Context<T> {
    async fn call(&mut self, request: Request<'_>) -> Result<Response, Error> {
        self.client.call(request).await
    }
}

#[async_trait]
impl<T: Client> Reader for Context<T> {
    async fn read_bits<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<Bit>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        // 1. 发出请求
        let call_result: Response = self
            .client
            .call(Request::ReadBits(addr.as_ref().into(), cnt))
            .await?;

        match call_result {
            Response::ReadBits(bits) => Ok(bits),
            _ => unreachable!("Only ReadBits responses are expected"),
        }
    }
    async fn read_words<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<Word>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        // 1. 发出请求
        let call_result = self
            .client
            .call(Request::ReadWords(addr.as_ref().into(), cnt))
            .await?;
        match call_result {
            Response::ReadWords(words) => Ok(words),
            _ => unreachable!("Only ReadBits responses are expected"),
        }
    }
}

#[async_trait]
impl<T: Client> Writer for Context<T> {
    async fn write_multiple_bits<A>(&mut self, addr: &A, bits: &[Bit]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        // match self
        //     .client
        //     .call(Request::WriteMultipleBits(
        //         addr.as_ref().into(),
        //         Cow::Borrowed(bits),
        //     ))
        //     .await?
        // {
        //     Ok(Response::WriteMultipleBits) => Ok(Ok(())),
        //     Ok(_) => unreachable!("call() should reject mismatching responses"),
        //     Err(e) => Err(Error::from(e)), // 使用 Error::from(e) 手动转换
        // }

        // 1. 发出请求
        let call_result = self
            .client
            .call(Request::WriteMultipleBits(
                addr.as_ref().into(),
                Cow::Borrowed(bits),
            ))
            .await?;
        match call_result {
            Response::WriteMultipleBits() => Ok(()),
            _ => unreachable!("Only ReadBits responses are expected"),
        }
    }

    async fn write_multiple_word<A>(&mut self, addr: &A, words: &[Word]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
         // 1. 发出请求
         let call_result = self
         .client
         .call(Request::WriteMultipleWords(
             addr.as_ref().into(),
             Cow::Borrowed(words),
         ))
         .await?;
     match call_result {
         Response::WriteMultipleWords() => Ok(()),
         _ => unreachable!("Only ReadBits responses are expected"),
     }
    }
}

#[cfg(test)]
mod tests {
    // use futures_executor::block_on;

    // use crate::{Error, Result};

    // use super::*;
    // use crate::frame::SoftElementCode;
    // use std::{any::Any, io, sync::Mutex};

    // #[derive(Debug)]
    // pub(crate) struct ClientMock {
    //     // slave: Option<Slave>,
    //     last_request: Mutex<Option<Request<'static>>>,
    //     next_response: Option<Result<Response>>,
    //     response: Response,
    // }

    // impl Default for ClientMock {
    //     fn default() -> Self {
    //         ClientMock {
    //             last_request: Mutex::new(None),
    //             next_response: None,
    //             response: Response::ReadBits(Vec::new()), // 设定一个默认的 Response 变体
    //         }
    //     }
    // }

    // #[allow(dead_code)]
    // impl ClientMock {
    //     pub(crate) fn last_request(&self) -> &Mutex<Option<Request<'static>>> {
    //         &self.last_request
    //     }

    //     pub(crate) fn set_next_response(&mut self, next_response: Result<Response>) {
    //         self.next_response = Some(next_response);
    //     }
    // }

    // #[async_trait]
    // impl Client for ClientMock {
    //     async fn call(&mut self, request: Request<'_>) -> Result<Response> {
    //         *self.last_request.lock().unwrap() = Some(request.into_owned());
    //         match self.next_response.take().unwrap() {
    //             Ok(response) => Ok(response),
    //             Err(Error::Transport(err)) => {
    //                 Err(io::Error::new(err.kind(), format!("{err}")).into())
    //             }
    //             Err(err) => Err(err),
    //         }
    //     }

    //     async fn disconnect(&mut self) -> io::Result<()> {
    //         Ok(())
    //     }
    // }

    // #[test]
    // fn read_some_coils() {
    //     // 设置模拟响应，假设从设备返回 4 个线圈状态
    //     let response_coils = vec![true, false, true, true];
    //     let mut client_mock = ClientMock::default();
    //     client_mock.set_next_response(Ok(Ok(Response::ReadBits(response_coils.clone()))));

    //     // 使用 ClientMock 创建一个 Context 实例
    //     let mut context = Context {
    //         client: Box::new(client_mock),
    //     };

    //     // 调用 `read_coils` 方法，并检查结果
    //     let addr = 0x0001; // 示例地址
    //     let cnt = 4; // 请求 4 个线圈状态
    //     let code = crate::frame::SoftElementCode::X;
    //     let result = block_on(context.read_bits(addr, cnt, code));

    //     // 验证请求和响应
    //     assert!(result.is_ok());
    //     assert_eq!(result.unwrap(), Ok(response_coils));
    // }

    // #[test]
    // fn read_some_discrete_inputs() {
    //     // The protocol will always return entire bytes with, i.e.
    //     // a multiple of 8 coils.
    //     let response_inputs = [true, false, false, true, false, true, false, true];
    //     for num_inputs in 1..8 {
    //         let mut client = Box::<ClientMock>::default();
    //         client.set_next_response(Ok(Ok(Response::ReadDiscreteInputs(
    //             response_inputs.to_vec(),
    //         ))));
    //         let mut context = Context { client };
    //         context.set_slave(Slave(1));
    //         let inputs = futures::executor::block_on(context.read_discrete_inputs(1, num_inputs))
    //             .unwrap()
    //             .unwrap();
    //         assert_eq!(&response_inputs[0..num_inputs as usize], &inputs[..]);
    //     }
    // }
}
