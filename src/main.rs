use bytes::{BufMut, BytesMut};

pub type Address = u32;

pub(crate) type Bit = bool;

pub(crate) type Word = u16;

pub(crate) type Code = u8;

pub type Quantity = u16;

// 主功能码的枚举
enum FunctionCode {
    Read(ReadOperation),
    Write(WriteOperation),
}

// 读取操作的枚举及其子指令
enum ReadOperation {
    Word {
        Address: u32, // 使用u32表示，低24位为起始地址
        soft_element_code: SoftElementCode,
        Quantity: u16,
    },
    Bit {
        Address: u32, // 使用u32表示，低24位为起始地址
        soft_element_code: SoftElementCode,
        Quantity: u16,
    },
}

// 写入操作的枚举及其子指令
enum WriteOperation {
    Word {
        Address: u32, // 使用u32表示，低24位为起始地址
        soft_element_code: SoftElementCode,
        data: Vec<Word>, // 写入的数据
    },
    Bit {
        Address: u32, // 使用u32表示，低24位为起始地址
        soft_element_code: SoftElementCode,
        data: Vec<Bit>, // 写入的开关量数据
    },
}

// 软元件代码的枚举
#[repr(u8)] // 设置每个元件代码为1字节
#[derive(Copy, Clone)]
enum SoftElementCode {
    X = 0x9C,
    Y = 0x9D,
    D = 0xA8,
    M = 0x90,
    // 其他软元件代码可以继续添加
}

// 定义常量
const SOFT_ELEMENT_CODE_D: u8 = 0xA8;

// 为 FunctionCode 枚举实现转换为字节的方法
impl FunctionCode {
    fn to_bytes(&self) -> BytesMut {
        let mut buffer = BytesMut::new();

        match self {
            // 读取操作的字节序列化
            FunctionCode::Read(operation) => {
                buffer.put_u16_le(0x0401);
                match operation {
                    ReadOperation::Word {
                        Address,
                        soft_element_code,
                        Quantity,
                    } => {
                        buffer.put_u16_le(0x0000);
                        // 将u32的低24位分解为3个u8字节
                        buffer.put_u16_le((Address & 0xFFFF) as u16);
                        buffer.put_u8((Address >> 16) as u8); // 高位字节
                        buffer.put_u8(*soft_element_code as u8); // 软元件代码
                        buffer.put_u16_le(*Quantity); // 元件数量
                    }
                    ReadOperation::Bit {
                        Address,
                        soft_element_code,
                        Quantity,
                    } => {
                        buffer.put_u16_le(0x0001);
                        buffer.put_u16_le((Address & 0xFFFF) as u16);
                        buffer.put_u8((Address >> 16) as u8); // 高位字节
                        buffer.put_u8(*soft_element_code as u8);
                        buffer.put_u16_le(*Quantity);
                    }
                }
            }

            // 写入操作的字节序列化
            FunctionCode::Write(operation) => {
                buffer.put_u16_le(0x1401); // 假设写入的功能码是 0x02
                match operation {
                    WriteOperation::Word {
                        Address,
                        soft_element_code,
                        data,
                    } => {
                        buffer.put_u16_le(0x0000);
                        buffer.put_u16_le((Address & 0xFFFF) as u16);
                        buffer.put_u8((Address >> 16) as u8); // 高位字节
                        buffer.put_u8(*soft_element_code as u8);
                        // buffer.put_u16_le(*element_count);
                        for word in data {
                            buffer.put_u16(*word); // 写入的数据
                        }
                    }
                    WriteOperation::Bit {
                        Address,
                        soft_element_code,
                        // element_count,
                        data,
                    } => {
                        buffer.put_u16_le(0x0001);
                        buffer.put_u16_le((Address & 0xFFFF) as u16);
                        buffer.put_u8((Address >> 16) as u8); // 高位字节
                        buffer.put_u8(*soft_element_code as u8);
                        // buffer.put_u16_le(*element_count);
                        for bit in data {
                            buffer.put_u8(*bit as u8); // 写入的开关量数据
                        }
                    }
                }
            }
        }

        buffer
    }
}

fn main() {
    // 示例：创建一个读取操作的请求
    let read_request = FunctionCode::Read(ReadOperation::Word {
        Address: 0x001234, // 示例地址
        soft_element_code: SoftElementCode::D,
        Quantity: 10,
    });

    // 获取序列化后的字节
    let read_bytes = read_request.to_bytes();
    println!("{:x?}", read_bytes); // 输出字节数组

    // 示例：创建一个写入操作的请求
    let write_request = FunctionCode::Write(WriteOperation::Bit {
        Address: 0x000789, // 示例地址
        soft_element_code: SoftElementCode::Y,
        data: vec![true, false, true, true, false],
    });

    // 获取序列化后的字节
    let write_bytes = write_request.to_bytes();
    println!("{:x?}", write_bytes); // 输出字节数组
}
