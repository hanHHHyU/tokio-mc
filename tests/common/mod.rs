use std::net::SocketAddr;

use tokio::net::TcpListener;

#[cfg(feature = "server")]
pub mod fake_plc;
pub mod mock_client;

#[cfg(feature = "server")]
pub use fake_plc::FakePlc;
pub use mock_client::MockClient;

pub async fn create_test_listener() -> (TcpListener, SocketAddr) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind test listener to localhost");
    let addr = listener
        .local_addr()
        .expect("failed to get test listener address");
    (listener, addr)
}

pub mod test_data {
    pub fn valid_read_request() -> Vec<u8> {
        vec![
            0x50, 0x00, 0x00, 0xFF, 0xFF, 0x03, 0x00, 0x0C, 0x00, 0x10, 0x00, 0x01, 0x04, 0x00,
            0x00, 0x00, 0x00, 0x00, 0xA8, 0x0A, 0x00,
        ]
    }

    pub fn valid_write_request(data: &[u8]) -> Vec<u8> {
        let mut request = vec![
            0x50, 0x00, 0x00, 0xFF, 0xFF, 0x03, 0x00, 0x10, 0x00, 0x10, 0x00, 0x01, 0x14, 0x00,
            0x00, 0x00, 0x00, 0x00, 0xA8, 0x02, 0x00,
        ];
        request.extend(data);
        request
    }
}
