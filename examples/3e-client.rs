use std::net::SocketAddr;
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

    let result = context.read_i32s("D1000", 1).await?;
    println!("Read U8s response: {:?}", result);
    Ok(())
}
