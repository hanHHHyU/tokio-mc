use std::net::SocketAddr;
use tokio_mc::{client::sync::*, frame::Model, Error};

fn main() -> Result<(), Error> {
    let addr = "192.168.110.252:5000".parse::<SocketAddr>().unwrap();
    // 传递 TcpClient 实例来初始化同步 Context
    let mut context = tcp::connect(addr)?;
    context.set_plc_model(Model::Keyence);

    // let words: Vec<u16> = vec![10];
    // let _ = context.write_multiple_words("D0", &words)?;

    // let f32: Vec<f32> = vec![70.0];
    // let _ = context.write_f32s("D0", &f32)?;
    // println!("Read words response: {:?}", read_result);

    let reult = context.read_reconver_string("D1404", 10)?;
    println!("Read words response: {:?}", reult);

    Ok(())
}
