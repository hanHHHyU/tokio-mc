#[cfg(feature = "sync")]
pub mod sync;
#[cfg(feature = "tcp")]
pub mod tcp;

use async_trait::async_trait;
use std::{borrow::Cow, fmt::Debug};

use crate::address::{converter_for_model, AddressConverter};
use crate::frame::*;
use crate::Error;

#[async_trait]
pub trait Client: Send + Debug {
    /// Invokes a _MC_ function.
    async fn call(&mut self, request: Request<'_>) -> Result<Response, Error>;

    /// Disconnect the client connection.
    async fn disconnect(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[async_trait]
pub trait Reader: Client {
    async fn read_u8s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<u8>, Error>
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

    async fn read_u64s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<u64>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    async fn read_i64s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<i64>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    async fn read_f64s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<f64>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    async fn read_bools<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<bool>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;
}

#[async_trait]
pub trait Writer: Client {
    async fn write_u8s<A>(&mut self, addr: &A, u8s: &[u8]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized;

    async fn write_bools<A>(&mut self, addr: &A, bools: &'_ [bool]) -> Result<(), Error>
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
}

/// Asynchronous Modbus client context with generic transport
#[derive(Debug)]
pub struct Context<T: Client> {
    client: T,
    model: Model, // 新增字段
    address_converter: Box<dyn AddressConverter>,
}

impl<T: Client> Context<T> {
    pub fn new(client: T) -> Self {
        let model = Model::default();
        Self {
            client,
            model,
            address_converter: converter_for_model(model),
        }
    }

    /// 设置 PLC 型号
    pub fn set_plc_model(&mut self, model: Model) {
        self.model = model;
        self.address_converter = converter_for_model(model);
    }

    /// Disconnect the client connection
    pub async fn disconnect(&mut self) -> std::io::Result<()> {
        self.client.disconnect().await
    }

    fn process_address<A>(&self, addr: &A) -> Result<String, Error>
    where
        A: AsRef<str> + ?Sized,
    {
        self.address_converter.convert(addr.as_ref())
    }
}

fn response_kind(response: &Response) -> &'static str {
    match response {
        Response::ReadU8s(_) => "ReadU8s",
        Response::WriteU8s() => "WriteU8s",
        Response::ReadBits(_) => "ReadBits",
        Response::WriteBits() => "WriteBits",
    }
}

fn unexpected_response(expected: &'static str, response: Response) -> Error {
    Error::Protocol(ProtocolError::UnexpectedResponse {
        expected,
        actual: response_kind(&response),
    })
}

#[async_trait]
impl<T: Client> Client for Context<T> {
    async fn call(&mut self, request: Request<'_>) -> Result<Response, Error> {
        self.client.call(request).await
    }
}

#[async_trait]
impl<T: Client> Reader for Context<T> {
    async fn read_u8s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<u8>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        self.client
            .call(Request::ReadU8s(self.process_address(addr)?.into(), cnt))
            .await
            .map(|response| match response {
                Response::ReadU8s(u8s) => Ok(u8s),
                other => Err(unexpected_response("ReadU8s", other)),
            })
            .and_then(|result| result)
    }

    async fn read_u16s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<u16>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        // 读取u8数据，每个u16需要2个u8字节
        let u8_data = self.read_u8s(addr, cnt).await?;

        // 将u8数据转换为小端字节序的u16
        let mut u16_data = Vec::with_capacity(cnt as usize);
        for chunk in u8_data.chunks_exact(2) {
            let value = u16::from_le_bytes([chunk[0], chunk[1]]);
            u16_data.push(value);
        }

        Ok(u16_data)
    }

    async fn read_i16s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<i16>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        // 读取u8数据，每个i16需要2个u8字节
        let u8_data = self.read_u8s(addr, cnt).await?;

        // 将u8数据转换为小端字节序的i16
        let mut i16_data = Vec::with_capacity(cnt as usize);
        for chunk in u8_data.chunks_exact(2) {
            let value = i16::from_le_bytes([chunk[0], chunk[1]]);
            i16_data.push(value);
        }

        Ok(i16_data)
    }

    async fn read_u32s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<u32>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        // 读取u8数据，每个u32需要4个u8字节
        let u8_data = self.read_u8s(addr, cnt * 2).await?;

        // 将u8数据转换为小端字节序的u32
        let mut u32_data = Vec::with_capacity(cnt as usize);
        for chunk in u8_data.chunks_exact(4) {
            let value = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            u32_data.push(value);
        }

        Ok(u32_data)
    }

    async fn read_i32s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<i32>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        // 读取u8数据，每个i32需要4个u8字节
        let u8_data = self.read_u8s(addr, cnt * 2).await?;

        // 将u8数据转换为小端字节序的i32
        let mut i32_data = Vec::with_capacity(cnt as usize);
        for chunk in u8_data.chunks_exact(4) {
            let value = i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            i32_data.push(value);
        }

        Ok(i32_data)
    }

    async fn read_f32s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<f32>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        // 读取u8数据，每个f32需要4个u8字节
        let u8_data = self.read_u8s(addr, cnt * 2).await?;

        // 将u8数据转换为小端字节序的f32
        let mut f32_data = Vec::with_capacity(cnt as usize);
        for chunk in u8_data.chunks_exact(4) {
            let value = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            f32_data.push(value);
        }

        Ok(f32_data)
    }

    async fn read_u64s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<u64>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        // 读取u8数据，每个u64需要8个u8字节
        let u8_data = self.read_u8s(addr, cnt * 4).await?;

        // 将u8数据转换为小端字节序的u64
        let mut u64_data = Vec::with_capacity(cnt as usize);
        for chunk in u8_data.chunks_exact(8) {
            let value = u64::from_le_bytes([
                chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7],
            ]);
            u64_data.push(value);
        }

        Ok(u64_data)
    }

    async fn read_i64s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<i64>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        // 读取u8数据，每个i64需要8个u8字节
        let u8_data = self.read_u8s(addr, cnt * 4).await?;

        // 将u8数据转换为小端字节序的i64
        let mut i64_data = Vec::with_capacity(cnt as usize);
        for chunk in u8_data.chunks_exact(8) {
            let value = i64::from_le_bytes([
                chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7],
            ]);
            i64_data.push(value);
        }

        Ok(i64_data)
    }

    async fn read_f64s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<f64>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        // 读取u8数据，每个f64需要8个u8字节
        let u8_data = self.read_u8s(addr, cnt * 4).await?;

        // 将u8数据转换为小端字节序的f64
        let mut f64_data = Vec::with_capacity(cnt as usize);
        for chunk in u8_data.chunks_exact(8) {
            let value = f64::from_le_bytes([
                chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7],
            ]);
            f64_data.push(value);
        }

        Ok(f64_data)
    }

    async fn read_bools<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<bool>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        self.client
            .call(Request::ReadBits(self.process_address(addr)?.into(), cnt))
            .await
            .map(|response| match response {
                Response::ReadBits(u8s) => Ok(u8s),
                other => Err(unexpected_response("ReadBits", other)),
            })
            .and_then(|result| result)
    }
}

#[async_trait]
impl<T: Client> Writer for Context<T> {
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
            .map(|response| match response {
                Response::WriteU8s() => Ok(()),
                other => Err(unexpected_response("WriteU8s", other)),
            })
            .and_then(|result| result)
    }

    async fn write_bools<A>(&mut self, addr: &A, bools: &'_ [bool]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        self.client
            .call(Request::WriteBits(
                self.process_address(addr)?.into(),
                Cow::Borrowed(bools),
            ))
            .await
            .map(|response| match response {
                Response::WriteBits() => Ok(()),
                other => Err(unexpected_response("WriteBits", other)),
            })
            .and_then(|result| result)
    }

    async fn write_u16s<A>(&mut self, addr: &A, u16s: &[u16]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        // 将u16数据转换为小端字节序的u8
        let mut u8s = Vec::with_capacity(u16s.len() * 2);
        for &value in u16s {
            u8s.extend_from_slice(&value.to_le_bytes());
        }
        self.write_u8s(addr, &u8s).await
    }

    async fn write_i16s<A>(&mut self, addr: &A, i16s: &[i16]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        // 将i16数据转换为小端字节序的u8
        let mut u8s = Vec::with_capacity(i16s.len() * 2);
        for &value in i16s {
            u8s.extend_from_slice(&value.to_le_bytes());
        }
        self.write_u8s(addr, &u8s).await
    }

    async fn write_u32s<A>(&mut self, addr: &A, u32s: &[u32]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        // 将u32数据转换为小端字节序的u8
        let mut u8s = Vec::with_capacity(u32s.len() * 4);
        for &value in u32s {
            u8s.extend_from_slice(&value.to_le_bytes());
        }
        self.write_u8s(addr, &u8s).await
    }

    async fn write_i32s<A>(&mut self, addr: &A, i32s: &[i32]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        // 将i32数据转换为小端字节序的u8
        let mut u8s = Vec::with_capacity(i32s.len() * 4);
        for &value in i32s {
            u8s.extend_from_slice(&value.to_le_bytes());
        }
        self.write_u8s(addr, &u8s).await
    }

    async fn write_f32s<A>(&mut self, addr: &A, f32s: &[f32]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        // 将f32数据转换为小端字节序的u8
        let mut u8s = Vec::with_capacity(f32s.len() * 4);
        for &value in f32s {
            u8s.extend_from_slice(&value.to_le_bytes());
        }
        self.write_u8s(addr, &u8s).await
    }

    async fn write_u64s<A>(&mut self, addr: &A, u64s: &[u64]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        // 将u64数据转换为小端字节序的u8
        let mut u8s = Vec::with_capacity(u64s.len() * 8);
        for &value in u64s {
            u8s.extend_from_slice(&value.to_le_bytes());
        }
        self.write_u8s(addr, &u8s).await
    }

    async fn write_i64s<A>(&mut self, addr: &A, i64s: &[i64]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        // 将i64数据转换为小端字节序的u8
        let mut u8s = Vec::with_capacity(i64s.len() * 8);
        for &value in i64s {
            u8s.extend_from_slice(&value.to_le_bytes());
        }
        self.write_u8s(addr, &u8s).await
    }

    async fn write_f64s<A>(&mut self, addr: &A, f64s: &[f64]) -> Result<(), Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        // 将f64数据转换为小端字节序的u8
        let mut u8s = Vec::with_capacity(f64s.len() * 8);
        for &value in f64s {
            u8s.extend_from_slice(&value.to_le_bytes());
        }
        self.write_u8s(addr, &u8s).await
    }
}

#[cfg(test)]
mod context_tests {
    use super::*;
    use crate::frame::{Model, Request, Response, LIMIT};
    use crate::test_common::MockClient;
    use crate::Error;

    fn assert_write_u8s_request(
        ctx: &Context<MockClient>,
        expected_addr: &str,
        expected_data: &[u8],
    ) {
        let request = ctx
            .client
            .requests()
            .last()
            .expect("expected a request to be recorded");
        match request {
            Request::WriteU8s(addr, data) => {
                assert_eq!(addr.as_ref(), expected_addr);
                assert_eq!(data.as_ref(), expected_data);
            }
            other => panic!("expected WriteU8s request, got {other:?}"),
        }
    }

    fn assert_write_bits_request(
        ctx: &Context<MockClient>,
        expected_addr: &str,
        expected_bits: &[bool],
    ) {
        let request = ctx
            .client
            .requests()
            .last()
            .expect("expected a request to be recorded");
        match request {
            Request::WriteBits(addr, bits) => {
                assert_eq!(addr.as_ref(), expected_addr);
                assert_eq!(bits.as_ref(), expected_bits);
            }
            other => panic!("expected WriteBits request, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_context_read_u8s() {
        let mock = MockClient::new().expect_response(Response::ReadU8s(vec![0x12, 0x34]));
        let mut ctx = Context::new(mock);

        let result = ctx.read_u8s("D100", 2).await;

        assert_eq!(result.unwrap(), vec![0x12, 0x34]);
        ctx.client.verify();
    }

    #[tokio::test]
    async fn test_context_read_u8s_empty() {
        let mock = MockClient::new().expect_response(Response::ReadU8s(Vec::new()));
        let mut ctx = Context::new(mock);

        let result = ctx.read_u8s("D100", 0).await;

        assert_eq!(result.unwrap(), Vec::<u8>::new());
        ctx.client.verify();
    }

    #[tokio::test]
    async fn test_context_read_u8s_max() {
        let expected = vec![0xAB; LIMIT as usize];
        let mock = MockClient::new().expect_response(Response::ReadU8s(expected.clone()));
        let mut ctx = Context::new(mock);

        let result = ctx.read_u8s("D100", LIMIT).await;

        assert_eq!(result.unwrap(), expected);
        ctx.client.verify();
    }

    #[tokio::test]
    async fn test_context_read_u16s() {
        let mock =
            MockClient::new().expect_response(Response::ReadU8s(vec![0x34, 0x12, 0x78, 0x56]));
        let mut ctx = Context::new(mock);

        let result = ctx.read_u16s("D100", 2).await;

        assert_eq!(result.unwrap(), vec![0x1234, 0x5678]);
        ctx.client.verify();
    }

    #[tokio::test]
    async fn test_context_read_i16s() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(-2i16).to_le_bytes());
        bytes.extend_from_slice(&(42i16).to_le_bytes());
        let mock = MockClient::new().expect_response(Response::ReadU8s(bytes));
        let mut ctx = Context::new(mock);

        let result = ctx.read_i16s("D100", 2).await;

        assert_eq!(result.unwrap(), vec![-2, 42]);
        ctx.client.verify();
    }

    #[tokio::test]
    async fn test_context_read_u32s() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&0x12345678u32.to_le_bytes());
        bytes.extend_from_slice(&0x90ABCDEFu32.to_le_bytes());
        let mock = MockClient::new().expect_response(Response::ReadU8s(bytes));
        let mut ctx = Context::new(mock);

        let result = ctx.read_u32s("D100", 2).await;

        assert_eq!(result.unwrap(), vec![0x12345678, 0x90ABCDEF]);
        ctx.client.verify();
    }

    #[tokio::test]
    async fn test_context_read_i32s() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(-2i32).to_le_bytes());
        bytes.extend_from_slice(&(42i32).to_le_bytes());
        let mock = MockClient::new().expect_response(Response::ReadU8s(bytes));
        let mut ctx = Context::new(mock);

        let result = ctx.read_i32s("D100", 2).await;

        assert_eq!(result.unwrap(), vec![-2, 42]);
        ctx.client.verify();
    }

    #[tokio::test]
    async fn test_context_read_f32s() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&1.5f32.to_le_bytes());
        bytes.extend_from_slice(&(-2.5f32).to_le_bytes());
        let mock = MockClient::new().expect_response(Response::ReadU8s(bytes));
        let mut ctx = Context::new(mock);

        let result = ctx.read_f32s("D100", 2).await;

        assert_eq!(result.unwrap(), vec![1.5, -2.5]);
        ctx.client.verify();
    }

    #[tokio::test]
    async fn test_context_read_u64s() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&0x1122334455667788u64.to_le_bytes());
        bytes.extend_from_slice(&0x99AABBCCDDEEFF00u64.to_le_bytes());
        let mock = MockClient::new().expect_response(Response::ReadU8s(bytes));
        let mut ctx = Context::new(mock);

        let result = ctx.read_u64s("D100", 2).await;

        assert_eq!(
            result.unwrap(),
            vec![0x1122334455667788, 0x99AABBCCDDEEFF00]
        );
        ctx.client.verify();
    }

    #[tokio::test]
    async fn test_context_read_i64s() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(-2i64).to_le_bytes());
        bytes.extend_from_slice(&(42i64).to_le_bytes());
        let mock = MockClient::new().expect_response(Response::ReadU8s(bytes));
        let mut ctx = Context::new(mock);

        let result = ctx.read_i64s("D100", 2).await;

        assert_eq!(result.unwrap(), vec![-2, 42]);
        ctx.client.verify();
    }

    #[tokio::test]
    async fn test_context_read_f64s() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&1.5f64.to_le_bytes());
        bytes.extend_from_slice(&(-2.5f64).to_le_bytes());
        let mock = MockClient::new().expect_response(Response::ReadU8s(bytes));
        let mut ctx = Context::new(mock);

        let result = ctx.read_f64s("D100", 2).await;

        assert_eq!(result.unwrap(), vec![1.5, -2.5]);
        ctx.client.verify();
    }

    #[tokio::test]
    async fn test_context_read_bools() {
        let mock = MockClient::new().expect_response(Response::ReadBits(vec![true, false, true]));
        let mut ctx = Context::new(mock);

        let result = ctx.read_bools("M10", 3).await;

        assert_eq!(result.unwrap(), vec![true, false, true]);
        ctx.client.verify();
    }

    #[tokio::test]
    async fn test_context_write_u8s() {
        let mock = MockClient::new().expect_response(Response::WriteU8s());
        let mut ctx = Context::new(mock);
        let data = vec![0x12, 0x34];

        let result = ctx.write_u8s("D100", &data).await;

        assert!(result.is_ok());
        assert_write_u8s_request(&ctx, "D100", &data);
        ctx.client.verify();
    }

    #[tokio::test]
    async fn test_context_write_u16s() {
        let mock = MockClient::new().expect_response(Response::WriteU8s());
        let mut ctx = Context::new(mock);
        let values = vec![0x1234u16, 0x5678u16];
        let expected = vec![0x34, 0x12, 0x78, 0x56];

        let result = ctx.write_u16s("D100", &values).await;

        assert!(result.is_ok());
        assert_write_u8s_request(&ctx, "D100", &expected);
        ctx.client.verify();
    }

    #[tokio::test]
    async fn test_context_write_i16s() {
        let mock = MockClient::new().expect_response(Response::WriteU8s());
        let mut ctx = Context::new(mock);
        let values = vec![-2i16, 42i16];
        let mut expected = Vec::new();
        expected.extend_from_slice(&(-2i16).to_le_bytes());
        expected.extend_from_slice(&(42i16).to_le_bytes());

        let result = ctx.write_i16s("D100", &values).await;

        assert!(result.is_ok());
        assert_write_u8s_request(&ctx, "D100", &expected);
        ctx.client.verify();
    }

    #[tokio::test]
    async fn test_context_write_u32s() {
        let mock = MockClient::new().expect_response(Response::WriteU8s());
        let mut ctx = Context::new(mock);
        let values = vec![0x12345678u32, 0x90ABCDEFu32];
        let mut expected = Vec::new();
        expected.extend_from_slice(&0x12345678u32.to_le_bytes());
        expected.extend_from_slice(&0x90ABCDEFu32.to_le_bytes());

        let result = ctx.write_u32s("D100", &values).await;

        assert!(result.is_ok());
        assert_write_u8s_request(&ctx, "D100", &expected);
        ctx.client.verify();
    }

    #[tokio::test]
    async fn test_context_write_i32s() {
        let mock = MockClient::new().expect_response(Response::WriteU8s());
        let mut ctx = Context::new(mock);
        let values = vec![-2i32, 42i32];
        let mut expected = Vec::new();
        expected.extend_from_slice(&(-2i32).to_le_bytes());
        expected.extend_from_slice(&(42i32).to_le_bytes());

        let result = ctx.write_i32s("D100", &values).await;

        assert!(result.is_ok());
        assert_write_u8s_request(&ctx, "D100", &expected);
        ctx.client.verify();
    }

    #[tokio::test]
    async fn test_context_write_f32s() {
        let mock = MockClient::new().expect_response(Response::WriteU8s());
        let mut ctx = Context::new(mock);
        let values = vec![1.5f32, -2.5f32];
        let mut expected = Vec::new();
        expected.extend_from_slice(&1.5f32.to_le_bytes());
        expected.extend_from_slice(&(-2.5f32).to_le_bytes());

        let result = ctx.write_f32s("D100", &values).await;

        assert!(result.is_ok());
        assert_write_u8s_request(&ctx, "D100", &expected);
        ctx.client.verify();
    }

    #[tokio::test]
    async fn test_context_write_u64s() {
        let mock = MockClient::new().expect_response(Response::WriteU8s());
        let mut ctx = Context::new(mock);
        let values = vec![0x1122334455667788u64, 0x99AABBCCDDEEFF00u64];
        let mut expected = Vec::new();
        expected.extend_from_slice(&0x1122334455667788u64.to_le_bytes());
        expected.extend_from_slice(&0x99AABBCCDDEEFF00u64.to_le_bytes());

        let result = ctx.write_u64s("D100", &values).await;

        assert!(result.is_ok());
        assert_write_u8s_request(&ctx, "D100", &expected);
        ctx.client.verify();
    }

    #[tokio::test]
    async fn test_context_write_i64s() {
        let mock = MockClient::new().expect_response(Response::WriteU8s());
        let mut ctx = Context::new(mock);
        let values = vec![-2i64, 42i64];
        let mut expected = Vec::new();
        expected.extend_from_slice(&(-2i64).to_le_bytes());
        expected.extend_from_slice(&(42i64).to_le_bytes());

        let result = ctx.write_i64s("D100", &values).await;

        assert!(result.is_ok());
        assert_write_u8s_request(&ctx, "D100", &expected);
        ctx.client.verify();
    }

    #[tokio::test]
    async fn test_context_write_f64s() {
        let mock = MockClient::new().expect_response(Response::WriteU8s());
        let mut ctx = Context::new(mock);
        let values = vec![1.5f64, -2.5f64];
        let mut expected = Vec::new();
        expected.extend_from_slice(&1.5f64.to_le_bytes());
        expected.extend_from_slice(&(-2.5f64).to_le_bytes());

        let result = ctx.write_f64s("D100", &values).await;

        assert!(result.is_ok());
        assert_write_u8s_request(&ctx, "D100", &expected);
        ctx.client.verify();
    }

    #[tokio::test]
    async fn test_context_write_bools() {
        let mock = MockClient::new().expect_response(Response::WriteBits());
        let mut ctx = Context::new(mock);
        let values = vec![true, false, true];

        let result = ctx.write_bools("M10", &values).await;

        assert!(result.is_ok());
        assert_write_bits_request(&ctx, "M10", &values);
        ctx.client.verify();
    }

    #[tokio::test]
    async fn test_context_process_address_mitsubishi() {
        let mock = MockClient::new().expect_response(Response::ReadU8s(vec![0x00, 0x00]));
        let mut ctx = Context::new(mock);
        ctx.set_plc_model(Model::Mitsubishi);

        let result = ctx.read_u16s("D100", 1).await;

        assert!(result.is_ok());
        let request = ctx.client.requests().first().expect("missing request");
        match request {
            Request::ReadU8s(addr, _) => assert_eq!(addr.as_ref(), "D100"),
            other => panic!("expected ReadU8s request, got {other:?}"),
        }
        ctx.client.verify();
    }

    #[tokio::test]
    async fn test_context_process_address_keyence() {
        let mock = MockClient::new().expect_response(Response::ReadU8s(vec![0x00, 0x00]));
        let mut ctx = Context::new(mock);
        ctx.set_plc_model(Model::Keyence);

        let result = ctx.read_u16s("DM100", 1).await;

        assert!(result.is_ok());
        let request = ctx.client.requests().first().expect("missing request");
        match request {
            Request::ReadU8s(addr, _) => assert_eq!(addr.as_ref(), "D100"),
            other => panic!("expected ReadU8s request, got {other:?}"),
        }
        ctx.client.verify();
    }

    #[tokio::test]
    async fn test_context_set_plc_model() {
        let mock = MockClient::new().expect_response(Response::ReadU8s(vec![0x00, 0x00]));
        let mut ctx = Context::new(mock);

        ctx.set_plc_model(Model::Keyence);
        assert!(matches!(ctx.model, Model::Keyence));
        ctx.set_plc_model(Model::Mitsubishi);
        assert!(matches!(ctx.model, Model::Mitsubishi));

        let _ = ctx.read_u16s("D100", 1).await;
        ctx.client.verify();
    }

    #[tokio::test]
    async fn test_context_invalid_address_returns_error() {
        let mock = MockClient::new();
        let mut ctx = Context::new(mock);
        ctx.set_plc_model(Model::Keyence);

        let result = ctx.read_u16s("INVALID", 1).await;

        match result {
            Err(Error::KV(_)) => {}
            other => panic!("expected KV error, got {other:?}"),
        }
        assert!(ctx.client.requests().is_empty());
    }
}
