use std::net::SocketAddr;
use tokio_mc::{
    client::{sync::*, Context},Error
};


fn main() -> Result<(), Error> {
    // 设置目标服务器地址
    let addr = "192.168.110.210:5000".parse::<SocketAddr>().unwrap();
    // 传递 TcpClient 实例来初始化同步 Context
    let mut context = tcp::connect(addr)?;

    // 读取 10 个字的示例
    let read_result = context.read_words("D0", 10)?;
    println!("Read words response: {:?}", read_result);

    // // 调用其他同步方法
    // let response = context.call(your_request_here)?; // 替换为实际请求
    // println!("Call response: {:?}", response);

    Ok(())
}
