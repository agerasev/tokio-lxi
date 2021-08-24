use std::net::{IpAddr, SocketAddr};

use tokio::io as tio;
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

pub struct DummyEmulator {
    listener: TcpListener,
}

impl DummyEmulator {
    pub async fn start(addr: IpAddr) -> Self {
        let listener = TcpListener::bind(&SocketAddr::new(addr, 0)).await.unwrap();
        DummyEmulator { listener }
    }

    pub fn address(&self) -> tio::Result<SocketAddr> {
        self.listener.local_addr()
    }

    pub async fn run(&mut self, conns: u64) {
        for _ in 0..conns {
            let (mut conn, _): (TcpStream, _) = self.listener.accept().await.unwrap();
            let (reader, mut writer) = conn.split();
            let mut reader = BufReader::new(reader);
            let mut command = vec![];
            reader.read_until(b'\n', &mut command).await.unwrap();

            let response = if command.starts_with(b"IDN?") {
                b"DummyEmulator".to_vec()
            } else {
                b"Error".to_vec()
            };

            writer.write_all(&response).await.unwrap();
            writer.write_all(b"\r\n").await.unwrap();
        }
    }
}
