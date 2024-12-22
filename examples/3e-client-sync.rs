use std::net::SocketAddr;
use tokio_mc::{client::sync::*, frame::Model, Error};

fn main() -> Result<(), Error> {
    let addr = "192.168.110.252:5000".parse::<SocketAddr>().unwrap();
    // 传递 TcpClient 实例来初始化同步 Context
    let mut context = tcp::connect(addr)?;
    context.set_plc_model(Model::Keyence);

    // let words: Vec<u16> = vec![10];
    // let _ = context.write_multiple_words("D0", &words)?;

    let read_result = context.read_f64s("D0", 1)?;
    println!("Read words response: {:?}", read_result);
    Ok(())
}
