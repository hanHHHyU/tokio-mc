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

    /// Disconnect the client connection
    pub async fn disconnect(&mut self) -> std::io::Result<()> {
        self.client.disconnect().await
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
    async fn read_u8s<A>(&mut self, addr: &A, cnt: Quantity) -> Result<Vec<u8>, Error>
    where
        A: AsRef<str> + Send + Sync + ?Sized,
    {
        self.client
            .call(Request::ReadU8s(self.process_address(addr)?.into(), cnt))
            .await
            .map(|response| match response {
                Response::ReadU8s(u8s) => Ok(u8s),
                _ => {
                    unreachable!("Unexpected response type, expected ReadU8s")
                }
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
                _ => {
                    unreachable!("Unexpected response type, expected ReadBits")
                }
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
                _ => unreachable!("Unexpected response type, expected WriteU8s"),
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
                _ => unreachable!("Unexpected response type, expected WriteBits"),
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
mod tests {}
