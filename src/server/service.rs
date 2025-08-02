use std::{future::Future, sync::Arc};

/// `Service` trait
pub trait Service {
    type Request;
    type Response;
    type Exception;
    type Future: Future<Output = Result<Self::Response, Self::Exception>> + Send;

    fn call(&self, req: Self::Request) -> Self::Future;
}

// Arc<T>的Service实现，允许在Arc中使用Service
impl<T> Service for Arc<T>
where
    T: Service,
{
    type Request = T::Request;
    type Response = T::Response;
    type Exception = T::Exception;
    type Future = T::Future;

    fn call(&self, req: Self::Request) -> Self::Future {
        (**self).call(req)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        borrow::Cow,
        collections::HashMap,
        future,
        sync::{Arc, Mutex},
    };

    use super::*;
    use crate::frame::{ProtocolError, Request, Response};
    use log;

    // 定义存储数据的结构
    struct ExampleService {
        d_registers: Arc<Mutex<HashMap<u16, u16>>>,
    }

    impl ExampleService {
        fn new() -> Self {
            Self {
                d_registers: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    impl Service for ExampleService {
        type Request = Request<'static>;
        type Response = Response;
        type Exception = ProtocolError;
        type Future = future::Ready<Result<Self::Response, Self::Exception>>;

        fn call(&self, req: Self::Request) -> Self::Future {
            let res = match req {
                Request::ReadU8s(ref addr, _) => {
                    let registers = self.d_registers.lock().unwrap();
                    let value = registers
                        .get(&addr.parse::<u16>().unwrap_or(0))
                        .cloned()
                        .unwrap_or(0);
                    // 将u16转换为u8数组
                    let bytes = value.to_le_bytes();
                    Ok(Response::ReadU8s(bytes.to_vec()))
                }
                Request::WriteU8s(ref addr, ref values) => {
                    let mut registers = self.d_registers.lock().unwrap();
                    // 从u8数组重构u16值（假设至少有2个字节）
                    let value = if values.len() >= 2 {
                        u16::from_le_bytes([values[0], values[1]])
                    } else {
                        values[0] as u16
                    };
                    registers.insert(addr.parse::<u16>().unwrap_or(0), value);
                    Ok(Response::WriteU8s())
                }
            };
            future::ready(res)
        }
    }

    /// **测试 ReadU8s**
    #[tokio::test]
    async fn test_read_u8s() {
        let service = ExampleService::new();
        service.d_registers.lock().unwrap().insert(100, 42);

        let request = Request::ReadU8s("100".into(), 2);
        let result = service.call(request).await;

        // 验证结果
        if let Ok(Response::ReadU8s(values)) = result {
            // 42的小端字节序应该是[42, 0]
            assert_eq!(values, vec![42, 0]);
            // Debug output in test
            log::debug!("读取到的值: {:?}", values);
        } else {
            panic!("调用出错: {:?}", result);
        }
    }

    #[tokio::test]
    async fn test_write_u8s() {
        let service = ExampleService::new();

        // 写入数据：55的小端字节序
        let request = Request::WriteU8s(Cow::Borrowed("100"), Cow::Borrowed(&[55, 0]));
        let result = service.call(request).await;
        assert!(result.is_ok());

        // 验证数据被正确写入
        let value = service.d_registers.lock().unwrap().get(&100).copied();
        assert_eq!(value, Some(55));

        // 读取验证
        let request = Request::ReadU8s("100".into(), 2);
        let result = service.call(request).await;

        if let Ok(Response::ReadU8s(values)) = result {
            assert_eq!(values, vec![55, 0]);
        } else {
            panic!("读取验证失败: {:?}", result);
        }
    }
}
