use std::net::SocketAddr;
use std::time::Duration;
use tokio_mc::{client::sync::*, frame::Model, Error};

fn main() -> Result<(), Error> {
    let addr = "127.0.0.1:9000".parse::<SocketAddr>().unwrap();
    
    // Use connection with timeout (5s connection timeout, 2s operation timeout)
    let mut context = tcp::connect_with_timeout(
        addr,
        Duration::from_secs(5),      // Connection timeout
        Some(Duration::from_secs(2)) // Operation timeout
    )?;
    
    println!("Connected successfully (5s connection timeout, 2s operation timeout)");
    
    context.set_plc_model(Model::Keyence);
    
    // Read value from D0
    let result = context.read_u8s("D0", 1)?;
    println!("Read words response: {:?}", result);
    
    // Read value from D100
    let response = context.read_u16s("D100", 1)?;
    println!("D100 = {:?}", response);
    
    // Test other operations
    context.write_u16s("D10", &[100, 200])?;
    println!("Write to D10-D11: [100, 200]");
    
    let values = context.read_u16s("D10", 2)?;
    println!("Read from D10-D11: {:?}", values);
    
    Ok(())
}