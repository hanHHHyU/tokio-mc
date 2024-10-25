use async_trait::async_trait;
use std::{borrow::Cow, fmt::Debug, io};

use crate::{frame::*, Result};

#[async_trait]
pub trait Client: Send + Debug {
    /// Invokes a _Modbus_ function.
    async fn call(&mut self, request: Request<'_>) -> Result<Response>;

    /// Disconnects the client.
    ///
    /// Permanently disconnects the client by shutting down the
    /// underlying stream in a graceful manner (`AsyncDrop`).
    ///
    /// Dropping the client without explicitly disconnecting it
    /// beforehand should also work and free all resources. The
    /// actual behavior might depend on the underlying transport
    /// protocol (RTU/TCP) that is used by the client.
    async fn disconnect(&mut self) -> io::Result<()>;
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

/// Asynchronous Modbus client context
#[derive(Debug)]
pub struct Context {
    client: Box<dyn Client>,
}

impl From<Box<dyn Client>> for Context {
    fn from(client: Box<dyn Client>) -> Self {
        Self { client }
    }
}

impl From<Context> for Box<dyn Client> {
    fn from(val: Context) -> Self {
        val.client
    }
}

#[async_trait]
impl Client for Context {
    async fn call(&mut self, request: Request<'_>) -> Result<Response> {
        self.client.call(request).await
    }

    async fn disconnect(&mut self) -> io::Result<()> {
        self.client.disconnect().await
    }
}
