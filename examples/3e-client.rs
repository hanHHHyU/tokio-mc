use std::net::SocketAddr;
// use tokio::io;
use tokio::time::Duration;
use tokio_mc::{
    client::{tcp::*, Reader},
    frame::Model,
    Error,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let addr = "192.168.1.30:5000"
        .parse::<SocketAddr>()
        .map_err(|e| Error::Transport(std::io::Error::new(std::io::ErrorKind::InvalidInput, e)))?;

    let mut context = connect(addr).await?;

    context.set_plc_model(Model::Keyence);

    // // 调用 read_bits 方法
    // let result = context.read_reconver_string("D1404", 10).await?;
    // println!("Read String response: {:?}", result);

    // let result = context.read_u8s("D1404", 10).await?;
    // println!("Read U8s response: {:?}", result);

    // loop {
    let result = context.read_i32s("D1000", 1).await?;
    println!("Read U8s response: {:?}", result);

    // 暂停 1 秒
    tokio::time::sleep(Duration::from_secs(1)).await;
    // }
    Ok(())
}
