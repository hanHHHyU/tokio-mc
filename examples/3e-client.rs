use std::net::SocketAddr;
// use tokio::io;
use tokio_mc::{
    client::{tcp::*, Reader},
    frame::Model,
    Error,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let addr = "192.168.110.252:5000"
        .parse::<SocketAddr>()
        .map_err(|e| Error::Transport(std::io::Error::new(std::io::ErrorKind::InvalidInput, e)))?;

    let mut context = connect(addr).await?;

    context.set_plc_model(Model::Keyence);

    // 调用 read_bits 方法
    let result = context.read_f32s("D0", 1000).await?;
    println!("Read bits response: {:?}", result);

    Ok(())
}
