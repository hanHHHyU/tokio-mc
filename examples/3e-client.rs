use std::net::SocketAddr;
use tokio_mc::{
    client::{tcp::*, Writer},
    frame::Model,
    Error,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let addr = "127.0.0.1:9000"
        .parse::<SocketAddr>()
        .map_err(|e| Error::Transport(std::io::Error::new(std::io::ErrorKind::InvalidInput, e)))?;

    let mut context = connect(addr).await?;

    context.set_plc_model(Model::Keyence);

    let u8s_to_write = vec![0x12, 0x34];

    let result = context.write_u8s("D0", &u8s_to_write).await?;
    println!("Read U8s response: {:?}", result);
    Ok(())
}
