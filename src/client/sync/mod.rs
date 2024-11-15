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
    fn read_bits<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<Bit>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    fn read_words<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<Word>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;
}

pub trait Writer: Client {
    fn write_multiple_bits<A>(&mut self, addr: &A, bits: &'_ [Bit]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    fn write_multiple_words<A>(&mut self, addr: &A, words: &'_ [Word]) -> Result<(), Error>
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
}

impl<T: AsyncClient> Client for Context<T> {
    fn call(&mut self, request: Request<'_>) -> Result<Response, Error> {
        block_on_with_timeout(&self.runtime, self.timeout, self.async_ctx.call(request))
    }
}

impl<T: AsyncClient> Reader for Context<T> {
    fn read_bits<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<Bit>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        block_on_with_timeout(
            &self.runtime,
            self.timeout,
            self.async_ctx.read_bits(addr, cnt),
        )
    }

    fn read_words<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<Word>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        block_on_with_timeout(
            &self.runtime,
            self.timeout,
            self.async_ctx.read_words(addr, cnt),
        )
    }
}

impl<T: AsyncClient> Writer for Context<T> {
    fn write_multiple_bits<A>(&mut self, addr: &A, bits: &[Bit]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        block_on_with_timeout(
            &self.runtime,
            self.timeout,
            self.async_ctx.write_multiple_bits(addr, bits),
        )
    }

    fn write_multiple_words<A>(&mut self, addr: &A, words: &[Word]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        block_on_with_timeout(
            &self.runtime,
            self.timeout,
            self.async_ctx.write_multiple_words(addr, words),
        )
    }
}
