# tokio-mc

[![Crates.io](https://img.shields.io/crates/v/tokio-mc.svg)](https://crates.io/crates/tokio-mc)
[![Docs.rs](https://docs.rs/tokio-mc/badge.svg)](https://docs.rs/tokio-mc)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**tokio-mc** is a pure Rust library for Mitsubishi Communication (MC) protocol, built on top of [tokio](https://tokio.rs/).  

---

## Features

- **Async & Sync** communication with Mitsubishi and Keyence PLCs using the 3E frame protocol.  
- Easy integration with the `tokio` ecosystem for async programming.  


---

## Installation

Add the following to your `Cargo.toml` to use `tokio-mc` with the desired features:  

- **Async Feature (3e-async)**: For asynchronous communication  
- **Sync Feature (3e-sync)**: For synchronous communication  

### Example Dependency

```toml
# For async usage
tokio-mc = { version = "0.1.3", features = ["3e-async"] }

# For sync usage
tokio-mc = { version = "0.1.3", features = ["3e-sync"] }
```


### Async Example

Here's how to use the async features of `tokio-mc`:  

```rust,no_run
use std::net::SocketAddr;
use tokio::time::Duration;
use tokio_mc::{
    client::{tcp::*, Reader, Writer},
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
    let u16_values = context.read_u16s("D1002", 2).await?;
    println!("Read U16s: {:?}", u16_values);
    
    Ok(())
}

```


### Sync Example

Here's how to use the sync features of `tokio-mc`:  

```rust,no_run
use std::net::SocketAddr;
use tokio_mc::{
    client::sync::{tcp::*, Reader, Writer},
    frame::Model,
    Error,
};

fn main() -> Result<(), Error> {
    let addr = "192.168.110.252:5000".parse::<SocketAddr>().unwrap();
    let mut context = connect(addr)?;
    context.set_plc_model(Model::Keyence);

    // Read different data types
    let u8_values = context.read_u8s("D1000", 2)?;
    println!("Read U8s: {:?}", u8_values);


    context.write_bools("M200", &[true, false, true, true])?;
    println!("Written Bools successfully");

    Ok(())
}


```


## Disclaimer

When using this library for PLC communication, please first make sure that there is no abnormality in your connection. I used the 3E frame protocol, which has been tested with Keyence and Mitsubishi and used in actual projects. If you have any feedback or suggestions, please contact me via QQ email.

Some codes are referenced from[ tokio-modbus](https://github.com/slowtec/tokio-modbus)。

