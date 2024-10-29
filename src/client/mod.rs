// #[cfg(feature = "sync")]
pub mod sync;
pub mod tcp;

use async_trait::async_trait;
use std::fmt::Debug;

use crate::{frame::*, Result};

#[async_trait]
pub trait Client: Send + Debug {
    /// Invokes a _Modbus_ function.
    async fn call(&mut self, request: Request<'_>) -> Result<Response>;

}

#[async_trait]
pub trait Reader: Client {
    async fn read_bits(
        &mut self,
        addr: Address,
        cnt: Quantity,
        code: SoftElementCode,
    ) -> Result<Vec<Bit>>;

    async fn read_words(
        &mut self,
        addr: Address,
        cnt: Quantity,
        code: SoftElementCode,
    ) -> Result<Vec<Word>>;
}

#[async_trait]
pub trait Writer: Client {
    async fn write_multiple_bits(
        &mut self,
        addr: Address,
        bits: &'_ [Bit],
        code: SoftElementCode,
    ) -> Result<()>;

    async fn write_multiple_word(
        &mut self,
        addr: Address,
        words: &'_ [Word],
        code: SoftElementCode,
    ) -> Result<()>;
}

/// Asynchronous Modbus client context with generic transport
#[derive(Debug)]
pub struct Context<T: Client> {
    client: T,
}

impl<T: Client> Context<T> {
    pub  fn new(client: T) -> Self {
        Self { client }
    }
}

#[async_trait]
impl<T: Client> Client for Context<T> {
    async fn call(&mut self, request: Request<'_>) -> Result<Response> {
        self.client.call(request).await
    }
}

#[async_trait]
impl<T: Client> Reader for Context<T> {
    async fn read_bits(
        &mut self,
        addr: Address,
        cnt: Quantity,
        code: SoftElementCode,
    ) -> Result<Vec<Bit>> {
        // 1. 发出请求
        let call_result = self.client.call(Request::ReadBits(addr, cnt, code)).await;
        // 2. 处理 call 的结果
        let result = match call_result {
            Ok(res) => res,
            Err(e) => return Err(e.into()),
        };

        // 3. 确保响应是 `ReadBits` 类型并提取位数据
        let mut bits = match result {
            Ok(Response::ReadBits(bits)) => bits,
            _ => unreachable!("call() should reject mismatching responses"),
        };

        // 4. 截断数据到指定数量
        debug_assert!(bits.len() >= cnt.into());
        bits.truncate(cnt.into());

        // 5. 返回最终的位数据
        Ok(Ok(bits))
    }
    async fn read_words(
        &mut self,
        addr: Address,
        cnt: Quantity,
        code: SoftElementCode,
    ) -> Result<Vec<Word>> {
        // 1. 发出请求
        let call_result = self.client.call(Request::ReadWords(addr, cnt, code)).await;
        // 2. 处理 call 的结果
        let result = match call_result {
            Ok(res) => res,
            Err(e) => return Err(e.into()),
        };

        // 3. 确保响应是 `ReadBits` 类型并提取位数据
        let mut words = match result {
            Ok(Response::ReadWords(words)) => words,
            _ => unreachable!("call() should reject mismatching responses"),
        };

        // 4. 截断数据到指定数量
        debug_assert!(words.len() >= cnt.into());
        words.truncate(cnt.into());

        // 5. 返回最终的位数据
        Ok(Ok(words))
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
