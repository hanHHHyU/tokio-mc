use futures_util::future::Either;
use std::{future::Future, io, time::Duration};
use tokio::runtime::Runtime;

use crate::{frame::*, Error};

use super::{Client as AsyncClient, Context as AsyncContext, Reader as _, Writer as _};
#[cfg(feature = "sync")]
pub mod tcp;

fn block_on_with_timeout<T, E>(
    runtime: &tokio::runtime::Runtime, // 传入一个 Tokio 运行时
    timeout: Option<Duration>,         // 可选的超时时间
    task: impl Future<Output = std::result::Result<T, E>>, // 异步任务，返回 `Result<T, E>`
) -> std::result::Result<T, E>
// 返回 `Result<T, E>`，其中 E 支持从 `io::Error` 转换
where
    E: From<io::Error>, // 要求 E 支持从 `io::Error` 转换
{
    // 根据是否设置了超时决定处理的方式
    let task = if let Some(duration) = timeout {
        // 如果 `timeout` 是 `Some`，即设置了超时
        Either::Left(async move {
            // 使用 `tokio::time::timeout` 包装任务，超时后会返回错误
            tokio::time::timeout(duration, task)
                .await
                .unwrap_or_else(|elapsed| {
                    // 如果超时发生，返回一个 `TimedOut` 错误，并转换为 `E` 类型
                    Err(io::Error::new(io::ErrorKind::TimedOut, elapsed).into())
                })
        })
    } else {
        // 如果 `timeout` 为 `None`，直接执行任务
        Either::Right(task)
    };
    // 使用 `runtime.block_on` 执行任务，并等待完成或超时
    runtime.block_on(task)
}

pub trait Client {
    fn call(&mut self, req: Request<'_>) -> Result<Response, Error>;
}

pub trait Reader: Client {
    fn read_bools<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<bool>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    fn read_u16s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<u16>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    fn read_i16s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<i16>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    fn read_u32s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<u32>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    fn read_i32s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<i32>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    fn read_f32s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<f32>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    fn read_f64s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<f64>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    fn read_u64s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<u64>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    fn read_i64s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<i64>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    fn read_u8s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<u8>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    fn read_reconver_string<A>(&mut self, addr: &A, cnt: Quantity) -> Result<String, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    fn read_string<A>(&mut self, addr: &A, cnt: Quantity) -> Result<String, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    fn read_u8s_and_bools<A>(
        &mut self,
        addr: &A,
        cnt: Quantity,
    ) -> Result<(Vec<u8>, Vec<bool>), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;
}

pub trait Writer: Client {
    fn write_bools<A>(&mut self, addr: &A, bools: &'_ [bool]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    fn write_u16s<A>(&mut self, addr: &A, u16s: &'_ [u16]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    fn write_i16s<A>(&mut self, addr: &A, i16s: &'_ [i16]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    fn write_u32s<A>(&mut self, addr: &A, u32s: &[u32]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    fn write_i32s<A>(&mut self, addr: &A, i32s: &[i32]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    fn write_f32s<A>(&mut self, addr: &A, f32s: &[f32]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    fn write_u64s<A>(&mut self, addr: &A, u64s: &[u64]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    fn write_i64s<A>(&mut self, addr: &A, i64s: &[i64]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    fn write_f64s<A>(&mut self, addr: &A, f64s: &[f64]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    fn write_u8s<A>(&mut self, addr: &A, u8s: &[u8]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    fn write_string<A>(&mut self, addr: &A, s: &A) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    fn write_reconver_string<A>(&mut self, addr: &A, s: &A) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;
}

#[derive(Debug)]
pub struct Context<T: AsyncClient> {
    runtime: tokio::runtime::Runtime,
    async_ctx: AsyncContext<T>,
    timeout: Option<Duration>,
}

impl<T: AsyncClient> Context<T> {
    /// 构造函数，初始化 `Context`，包含 `runtime` 和 `async_ctx`
    pub fn new(async_ctx: T, runtime: Runtime, timeout: Option<Duration>) -> Self {
        // 将传入的 `async_ctx` 包装为 `AsyncContext`
        let async_ctx = AsyncContext::new(async_ctx); // 假设 `AsyncContext` 有 `new` 构造函数

        Self {
            async_ctx,
            runtime,
            timeout,
        }
    }

    pub fn set_plc_model(&mut self, model: Model) {
        // 将模型传递给异步上下文
        self.async_ctx.set_plc_model(model);
    }
}

impl<T: AsyncClient> Client for Context<T> {
    fn call(&mut self, request: Request<'_>) -> Result<Response, Error> {
        block_on_with_timeout(&self.runtime, self.timeout, self.async_ctx.call(request))
    }
}

impl<T: AsyncClient> Reader for Context<T> {
    fn read_bools<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<bool>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        block_on_with_timeout(
            &self.runtime,
            self.timeout,
            self.async_ctx.read_bools(addr, cnt),
        )
    }

    fn read_u16s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<u16>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        block_on_with_timeout(
            &self.runtime,
            self.timeout,
            self.async_ctx.read_u16s(addr, cnt),
        )
    }

    fn read_i16s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<i16>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        block_on_with_timeout(
            &self.runtime,
            self.timeout,
            self.async_ctx.read_i16s(addr, cnt),
        )
    }

    fn read_u32s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<u32>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        block_on_with_timeout(
            &self.runtime,
            self.timeout,
            self.async_ctx.read_u32s(addr, cnt),
        )
    }

    fn read_i32s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<i32>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        block_on_with_timeout(
            &self.runtime,
            self.timeout,
            self.async_ctx.read_i32s(addr, cnt),
        )
    }

    fn read_f32s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<f32>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        block_on_with_timeout(
            &self.runtime,
            self.timeout,
            self.async_ctx.read_f32s(addr, cnt),
        )
    }

    fn read_f64s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<f64>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        block_on_with_timeout(
            &self.runtime,
            self.timeout,
            self.async_ctx.read_f64s(addr, cnt),
        )
    }

    fn read_u64s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<u64>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        block_on_with_timeout(
            &self.runtime,
            self.timeout,
            self.async_ctx.read_u64s(addr, cnt),
        )
    }

    fn read_i64s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<i64>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        block_on_with_timeout(
            &self.runtime,
            self.timeout,
            self.async_ctx.read_i64s(addr, cnt),
        )
    }

    fn read_u8s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<u8>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        block_on_with_timeout(
            &self.runtime,
            self.timeout,
            self.async_ctx.read_u8s(addr, cnt),
        )
    }
    fn read_reconver_string<A>(&mut self, addr: &A, cnt: Quantity) -> Result<String, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        block_on_with_timeout(
            &self.runtime,
            self.timeout,
            self.async_ctx.read_reconver_string(addr, cnt),
        )
    }

    fn read_string<A>(&mut self, addr: &A, cnt: Quantity) -> Result<String, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        block_on_with_timeout(
            &self.runtime,
            self.timeout,
            self.async_ctx.read_string(addr, cnt),
        )
    }

    fn read_u8s_and_bools<A>(
        &mut self,
        addr: &A,
        cnt: Quantity,
    ) -> Result<(Vec<u8>, Vec<bool>), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        block_on_with_timeout(
            &self.runtime,
            self.timeout,
            self.async_ctx.read_u8s_and_bools(addr, cnt),
        )
    }
}

impl<T: AsyncClient> Writer for Context<T> {
    fn write_bools<A>(&mut self, addr: &A, bools: &[bool]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        block_on_with_timeout(
            &self.runtime,
            self.timeout,
            self.async_ctx.write_bools(addr, bools),
        )
    }

    fn write_u16s<A>(&mut self, addr: &A, u16s: &[u16]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        block_on_with_timeout(
            &self.runtime,
            self.timeout,
            self.async_ctx.write_u16s(addr, u16s),
        )
    }

    fn write_i16s<A>(&mut self, addr: &A, i16s: &[i16]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        block_on_with_timeout(
            &self.runtime,
            self.timeout,
            self.async_ctx.write_i16s(addr, i16s),
        )
    }

    fn write_u32s<A>(&mut self, addr: &A, u32s: &[u32]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        block_on_with_timeout(
            &self.runtime,
            self.timeout,
            self.async_ctx.write_u32s(addr, u32s),
        )
    }

    fn write_i32s<A>(&mut self, addr: &A, i32s: &[i32]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        block_on_with_timeout(
            &self.runtime,
            self.timeout,
            self.async_ctx.write_i32s(addr, i32s),
        )
    }

    fn write_f32s<A>(&mut self, addr: &A, f32s: &[f32]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        block_on_with_timeout(
            &self.runtime,
            self.timeout,
            self.async_ctx.write_f32s(addr, f32s),
        )
    }

    fn write_u64s<A>(&mut self, addr: &A, u64s: &[u64]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        block_on_with_timeout(
            &self.runtime,
            self.timeout,
            self.async_ctx.write_u64s(addr, u64s),
        )
    }

    fn write_i64s<A>(&mut self, addr: &A, i64s: &[i64]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        block_on_with_timeout(
            &self.runtime,
            self.timeout,
            self.async_ctx.write_i64s(addr, i64s),
        )
    }

    fn write_f64s<A>(&mut self, addr: &A, f64s: &[f64]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        block_on_with_timeout(
            &self.runtime,
            self.timeout,
            self.async_ctx.write_f64s(addr, f64s),
        )
    }

    fn write_u8s<A>(&mut self, addr: &A, u8s: &[u8]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        block_on_with_timeout(
            &self.runtime,
            self.timeout,
            self.async_ctx.write_u8s(addr, u8s),
        )
    }

    fn write_string<A>(&mut self, addr: &A, s: &A) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        block_on_with_timeout(
            &self.runtime,
            self.timeout,
            self.async_ctx.write_string(addr, s),
        )
    }

    fn write_reconver_string<A>(&mut self, addr: &A, s: &A) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        block_on_with_timeout(
            &self.runtime,
            self.timeout,
            self.async_ctx.write_reconver_string(addr, s),
        )
    }
}
