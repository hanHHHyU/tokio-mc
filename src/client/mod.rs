#[cfg(feature = "sync")]
pub mod sync;
#[cfg(feature = "tcp")]
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
    async fn read_bools<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<bool>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    async fn read_u16s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<u16>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    async fn read_i16s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<i16>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    async fn read_u32s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<u32>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    async fn read_i32s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<i32>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    async fn read_f32s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<f32>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    async fn read_f64s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<f64>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    async fn read_u64s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<u64>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    async fn read_i64s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<i64>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    async fn read_u8s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<u8>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    async fn read_string<A>(&mut self, addr: &A, cnt: Quantity) -> Result<String, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    async fn read_reconver_string<A>(&mut self, addr: &A, cnt: Quantity) -> Result<String, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;
    async fn read_u8s_and_bools<A>(
        &mut self,
        addr: &A,
        cnt: Quantity,
    ) -> Result<(Vec<u8>, Vec<bool>), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;
}

#[async_trait]
pub trait Writer: Client {
    async fn write_bools<A>(&mut self, addr: &A, bits: &[bool]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    async fn write_u16s<A>(&mut self, addr: &A, u16s: &[u16]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    async fn write_i16s<A>(&mut self, addr: &A, i16s: &[i16]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    async fn write_u32s<A>(&mut self, addr: &A, u32s: &[u32]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    async fn write_i32s<A>(&mut self, addr: &A, i32s: &[i32]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    async fn write_f32s<A>(&mut self, addr: &A, f32s: &[f32]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    async fn write_u64s<A>(&mut self, addr: &A, u64s: &[u64]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    async fn write_i64s<A>(&mut self, addr: &A, i64s: &[i64]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    async fn write_f64s<A>(&mut self, addr: &A, f64s: &[f64]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    async fn write_u8s<A>(&mut self, addr: &A, u8s: &[u8]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    async fn write_string<A>(&mut self, addr: &A, s: &A) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    async fn write_reconver_string<A>(&mut self, addr: &A, s: &A) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;
}

/// Asynchronous Modbus client context with generic transport
#[derive(Debug)]
pub struct Context<T: Client> {
    client: T,
    model: Model, // 新增字段
}

impl<T: Client> Context<T> {
    pub fn new(client: T) -> Self {
        Self {
            client,
            model: Model::default(), // 使用默认值
        }
    }

    /// 设置 PLC 型号
    pub fn set_plc_model(&mut self, model: Model) {
        self.model = model;
    }

    fn process_address<A>(&self, addr: &A) -> Result<String, Error>
    where
        A: AsRef<str> + ?Sized,
    {
        match self.model {
            Model::Keyence => {
                // 调用地址转换方法
                match convert_keyence_to_mitsubishi_address(addr.as_ref()) {
                    Ok(converted_addr) => Ok(converted_addr),
                    Err(e) => Err(Error::KV(e)),
                }
            }
            Model::Mitsubishi => {
                // Mitsubishi 不进行处理，直接返回地址
                Ok(addr.as_ref().to_string())
            }
        }
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
    async fn read_bools<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<bool>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        self.client
            .call(Request::ReadBools(self.process_address(addr)?.into(), cnt))
            .await
            .map(|response| {
                match response {
                    Response::ReadBools(bools) => Ok(bools),
                    _ => {
                        // 如果响应不是 `ReadI16s` 类型，则触发错误
                        unreachable!("Unexpected response type, expected ReadBools")
                    }
                }
            })
            .and_then(|result| result) // 进一步解包 `Result<Vec<i16>, Error>`
    }

    async fn read_u16s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<u16>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        // 调用 `self.client.call`，获取 `Response` 类型的结果
        self.client
            .call(Request::ReadU16s(self.process_address(addr)?.into(), cnt))
            .await
            .map(|response| {
                match response {
                    Response::ReadU16s(u16s) => Ok(u16s),
                    _ => {
                        // 如果响应不是 `ReadI16s` 类型，则触发错误
                        unreachable!("Unexpected response type, expected ReadU16s")
                    }
                }
            })
            .and_then(|result| result) // 进一步解包 `Result<Vec<i16>, Error>`
    }

    async fn read_i16s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<i16>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        self.client
            .call(Request::ReadI16s(self.process_address(addr)?.into(), cnt))
            .await
            .map(|response| {
                match response {
                    Response::ReadI16s(i16s) => Ok(i16s),
                    _ => {
                        // 如果响应不是 `ReadI16s` 类型，则触发错误
                        unreachable!("Unexpected response type, expected ReadI16s")
                    }
                }
            })
            .and_then(|result| result) // 进一步解包 `Result<Vec<i16>, Error>`
    }

    async fn read_u32s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<u32>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        self.client
            .call(Request::ReadU32s(self.process_address(addr)?.into(), cnt))
            .await
            .map(|response| {
                match response {
                    Response::ReadU32s(u32s) => Ok(u32s),
                    _ => {
                        // 如果响应不是 `ReadI16s` 类型，则触发错误
                        unreachable!("Unexpected response type, expected ReadU32s")
                    }
                }
            })
            .and_then(|result| result)
    }

    async fn read_i32s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<i32>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        self.client
            .call(Request::ReadI32s(self.process_address(addr)?.into(), cnt))
            .await
            .map(|response| {
                match response {
                    Response::ReadI32s(i32s) => Ok(i32s),
                    _ => {
                        // 如果响应不是 `ReadI16s` 类型，则触发错误
                        unreachable!("Unexpected response type, expected ReadI32s")
                    }
                }
            })
            .and_then(|result| result)
    }

    async fn read_f32s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<f32>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        self.client
            .call(Request::ReadF32s(self.process_address(addr)?.into(), cnt))
            .await
            .map(|response| {
                match response {
                    Response::ReadF32s(f32s) => Ok(f32s),
                    _ => {
                        // 如果响应不是 `ReadI16s` 类型，则触发错误
                        unreachable!("Unexpected response type, expected ReadI32s")
                    }
                }
            })
            .and_then(|result| result)
    }
    async fn read_f64s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<f64>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        self.client
            .call(Request::ReadF64s(self.process_address(addr)?.into(), cnt))
            .await
            .map(|response| {
                match response {
                    Response::ReadF64s(f64s) => Ok(f64s),
                    _ => {
                        // 如果响应不是 `ReadI16s` 类型，则触发错误
                        unreachable!("Unexpected response type, expected ReadI32s")
                    }
                }
            })
            .and_then(|result| result)
    }

    async fn read_u64s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<u64>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        self.client
            .call(Request::ReadU64s(self.process_address(addr)?.into(), cnt))
            .await
            .map(|response| {
                match response {
                    Response::ReadU64s(u64s) => Ok(u64s),
                    _ => {
                        // 如果响应不是 `ReadI16s` 类型，则触发错误
                        unreachable!("Unexpected response type, expected ReadU64s")
                    }
                }
            })
            .and_then(|result| result)
    }

    async fn read_i64s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<i64>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        self.client
            .call(Request::ReadI64s(self.process_address(addr)?.into(), cnt))
            .await
            .map(|response| {
                match response {
                    Response::ReadI64s(i64s) => Ok(i64s),
                    _ => {
                        // 如果响应不是 `ReadI16s` 类型，则触发错误
                        unreachable!("Unexpected response type, expected ReadI64s")
                    }
                }
            })
            .and_then(|result| result)
    }

    async fn read_u8s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<u8>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        self.client
            .call(Request::ReadU8s(self.process_address(addr)?.into(), cnt))
            .await
            .map(|response| {
                match response {
                    Response::ReadU8s(u8s) => Ok(u8s),
                    _ => {
                        // 如果响应不是 `ReadI16s` 类型，则触发错误
                        unreachable!("Unexpected response type, expected ReadU8s")
                    }
                }
            })
            .and_then(|result| result)
    }

    async fn read_reconver_string<A>(&mut self, addr: &A, cnt: Quantity) -> Result<String, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        self.client
            .call(Request::ReadReconverString(
                self.process_address(addr)?.into(),
                cnt,
            ))
            .await
            .map(|response| {
                match response {
                    Response::ReadReconverString(be_string) => Ok(be_string),
                    _ => {
                        // 如果响应不是 `ReadI16s` 类型，则触发错误
                        unreachable!("Unexpected response type, expected ReadReconverString")
                    }
                }
            })
            .and_then(|result| result)
    }

    async fn read_string<A>(&mut self, addr: &A, cnt: Quantity) -> Result<String, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        self.client
            .call(Request::ReadString(self.process_address(addr)?.into(), cnt))
            .await
            .map(|response| {
                match response {
                    Response::ReadString(be_string) => Ok(be_string),
                    _ => {
                        // 如果响应不是 `ReadI16s` 类型，则触发错误
                        unreachable!("Unexpected response type, expected ReadString")
                    }
                }
            })
            .and_then(|result| result)
    }

    async fn read_u8s_and_bools<A>(
        &mut self,
        addr: &A,
        cnt: Quantity,
    ) -> Result<(Vec<u8>, Vec<bool>), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        self.client
            .call(Request::ReadU8sAndBools(
                self.process_address(addr)?.into(),
                cnt,
            ))
            .await
            .map(|response| {
                match response {
                    Response::ReadU8sAndBools(u8s, bools) => Ok((u8s, bools)),
                    _ => {
                        // 如果响应不是 `ReadI16s` 类型，则触发错误
                        unreachable!("Unexpected response type, expected ReadU8sAndBools")
                    }
                }
            })
            .and_then(|result| result)
    }
}

#[async_trait]
impl<T: Client> Writer for Context<T> {
    async fn write_bools<A>(&mut self, addr: &A, bools: &[bool]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        self.client
            .call(Request::WriteBools(
                self.process_address(addr)?.into(),
                Cow::Borrowed(bools),
            ))
            .await
            .map(|response| {
                // 使用 `map` 进行链式调用，检查响应类型
                match response {
                    Response::WriteBools() => Ok(()),
                    _ => unreachable!("Unexpected response type, expected WriteBools"),
                }
            })
            .and_then(|result| result)
    }

    async fn write_u16s<A>(&mut self, addr: &A, u16s: &[u16]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        self.client
            .call(Request::WriteU16s(
                self.process_address(addr)?.into(),
                Cow::Borrowed(u16s),
            ))
            .await
            .map(|response| {
                // 使用 `map` 进行链式调用，检查响应类型
                match response {
                    Response::WriteU16s() => Ok(()),
                    _ => unreachable!("Unexpected response type, expected WriteU16s"),
                }
            })
            .and_then(|result| result)
    }

    async fn write_i16s<A>(&mut self, addr: &A, i16s: &[i16]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        self.client
            .call(Request::WriteI16s(
                self.process_address(addr)?.into(),
                Cow::Borrowed(i16s),
            ))
            .await
            .map(|response| {
                // 使用 `map` 进行链式调用，检查响应类型
                match response {
                    Response::WriteI16s() => Ok(()),
                    _ => unreachable!("Unexpected response type, expected WriteU16s"),
                }
            })
            .and_then(|result| result)
    }

    async fn write_u32s<A>(&mut self, addr: &A, u32s: &[u32]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        self.client
            .call(Request::WriteU32s(
                self.process_address(addr)?.into(),
                Cow::Borrowed(u32s),
            ))
            .await
            .map(|response| {
                // 使用 `map` 进行链式调用，检查响应类型
                match response {
                    Response::WriteU32s() => Ok(()),
                    _ => unreachable!("Unexpected response type, expected WriteU32s"),
                }
            })
            .and_then(|result| result)
    }

    async fn write_i32s<A>(&mut self, addr: &A, i32s: &[i32]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        self.client
            .call(Request::WriteI32s(
                self.process_address(addr)?.into(),
                Cow::Borrowed(i32s),
            ))
            .await
            .map(|response| {
                // 使用 `map` 进行链式调用，检查响应类型
                match response {
                    Response::WriteI32s() => Ok(()),
                    _ => unreachable!("Unexpected response type, expected WriteI32s"),
                }
            })
            .and_then(|result| result)
    }
    async fn write_f32s<A>(&mut self, addr: &A, f32s: &[f32]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        self.client
            .call(Request::WriteF32s(
                self.process_address(addr)?.into(),
                Cow::Borrowed(f32s),
            ))
            .await
            .map(|response| {
                // 使用 `map` 进行链式调用，检查响应类型
                match response {
                    Response::WriteF32s() => Ok(()),
                    _ => unreachable!("Unexpected response type, expected WriteF32s"),
                }
            })
            .and_then(|result| result)
    }

    async fn write_u64s<A>(&mut self, addr: &A, u64s: &[u64]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        self.client
            .call(Request::WriteU64s(
                self.process_address(addr)?.into(),
                Cow::Borrowed(u64s),
            ))
            .await
            .map(|response| {
                // 使用 `map` 进行链式调用，检查响应类型
                match response {
                    Response::WriteU64s() => Ok(()),
                    _ => unreachable!("Unexpected response type, expected WriteU64s"),
                }
            })
            .and_then(|result| result)
    }

    async fn write_i64s<A>(&mut self, addr: &A, i64s: &[i64]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        self.client
            .call(Request::WriteI64s(
                self.process_address(addr)?.into(),
                Cow::Borrowed(i64s),
            ))
            .await
            .map(|response| {
                // 使用 `map` 进行链式调用，检查响应类型
                match response {
                    Response::WriteI64s() => Ok(()),
                    _ => unreachable!("Unexpected response type, expected WriteI64s"),
                }
            })
            .and_then(|result| result)
    }

    async fn write_f64s<A>(&mut self, addr: &A, f64s: &[f64]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        self.client
            .call(Request::WriteF64s(
                self.process_address(addr)?.into(),
                Cow::Borrowed(f64s),
            ))
            .await
            .map(|response| {
                // 使用 `map` 进行链式调用，检查响应类型
                match response {
                    Response::WriteF64s() => Ok(()),
                    _ => unreachable!("Unexpected response type, expected WriteF64s"),
                }
            })
            .and_then(|result| result)
    }

    async fn write_u8s<A>(&mut self, addr: &A, u8s: &[u8]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        self.client
            .call(Request::WriteU8s(
                self.process_address(addr)?.into(),
                Cow::Borrowed(u8s),
            ))
            .await
            .map(|response| {
                // 使用 `map` 进行链式调用，检查响应类型
                match response {
                    Response::WriteU8s() => Ok(()),
                    _ => unreachable!("Unexpected response type, expected WriteU8s"),
                }
            })
            .and_then(|result| result)
    }

    async fn write_string<A>(&mut self, addr: &A, s: &A) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        self.client
            .call(Request::WriteString(
                self.process_address(addr)?.into(),
                s.as_ref().to_string(), // 显式转换为 String
            ))
            .await
            .map(|response| match response {
                Response::WriteString() => Ok(()),
                _ => unreachable!("Unexpected response type, expected WriteString"),
            })
            .and_then(|result| result)
    }
    async fn write_reconver_string<A>(&mut self, addr: &A, s: &A) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        self.client
            .call(Request::WriteReconverString(
                self.process_address(addr)?.into(),
                s.as_ref().to_string(), // 显式转换为 String
            ))
            .await
            .map(|response| match response {
                Response::WriteReconverString() => Ok(()),
                _ => unreachable!("Unexpected response type, expected WriteReconverString"),
            })
            .and_then(|result| result)
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
    //     let result = block_on(context.read_bools(addr, cnt, code));

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
