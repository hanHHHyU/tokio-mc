use std::net::SocketAddr;
use tokio_mc::{client::sync::*, frame::Model, Error};

fn main() -> Result<(), Error> {
    let addr = "127.0.0.1:9000".parse::<SocketAddr>().unwrap();
    let mut context = tcp::connect(addr)?;
    context.set_plc_model(Model::Keyence);

    let result = context.read_u8s("D0", 1)?;
    println!("Read words response: {:?}", result);

    Ok(())
}
