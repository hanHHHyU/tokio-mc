// use std::{io, time::Duration};
// use tokio::runtime::Runtime;

// use super::{Address, Bit, Client, Quantity, Request, Response, SoftElementCode, Word};

// use crate::{frame::*, Result};

// use super::{
//     Client as AsyncClient, Context as AsyncContext, Reader as _, Writer as _,
// };

// /// 同步超时执行器
// fn block_on_with_timeout<F, T>(
//     runtime: &Runtime,
//     duration: Option<Duration>,
//     future: F,
// ) -> io::Result<T>
// where
//     F: std::future::Future<Output = io::Result<T>>,
// {
//     runtime.block_on(async {
//         if let Some(timeout) = duration {
//             tokio::time::timeout(timeout, future)
//                 .await
//                 .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "Operation timed out"))?
//         } else {
//             future.await
//         }
//     })
// }

// /// 异步和同步 `Context` 实现
// impl<T: AsyncClient + Sync + Send> Context<T> {
//     /// 同步 `call` 方法，通过 `block_on_with_timeout` 运行异步 `call`
//     pub fn call(&mut self, req: Request<'_>) -> io::Result<Response> {
//         // 调用异步的 `call` 方法并使用 `block_on_with_timeout` 执行
//         block_on_with_timeout(&self.runtime, self.timeout, self.async_ctx.call(req))
//     }
// }
// /// 泛型 Context 结构体
// #[derive(Debug)]
// pub struct Context<T> {
//     runtime: Runtime,       // 异步运行时
//     async_ctx: T,           // 泛型异步上下文
//     timeout: Option<Duration>, // 可选的超时时间
// }

// impl<T> Context<T> {
//     /// 创建新的泛型 Context 实例
//     pub fn new(runtime: Runtime, async_ctx: T, timeout: Option<Duration>) -> Self {
//         Self { runtime, async_ctx, timeout }
//     }

//     /// 返回当前的超时时间
//     pub const fn timeout(&self) -> Option<Duration> {
//         self.timeout
//     }

//     /// 设置超时时间
//     pub fn set_timeout(&mut self, duration: impl Into<Option<Duration>>) {
//         self.timeout = duration.into();
//     }

//     /// 重置超时时间
//     pub fn reset_timeout(&mut self) {
//         self.timeout = None;
//     }
// }

// pub trait Reader: Client {
//      fn read_bits(
//         &mut self,
//         addr: Address,
//         cnt: Quantity,
//         code: SoftElementCode,
//     ) -> Result<Vec<Bit>>;

//      fn read_words(
//         &mut self,
//         addr: Address,
//         cnt: Quantity,
//         code: SoftElementCode,
//     ) -> Result<Vec<Word>>;
// }


// pub trait Writer: Client {
//      fn write_multiple_bits(
//         &mut self,
//         addr: Address,
//         bits: &'_ [Bit],
//         code: SoftElementCode,
//     ) -> Result<()>;

//      fn write_multiple_word(
//         &mut self,
//         addr: Address,
//         words: &'_ [Word],
//         code: SoftElementCode,
//     ) -> Result<()>;
// }