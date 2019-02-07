use std::io::{BufReader};
use std::net::{SocketAddr, IpAddr};

use tokio::prelude::*;
use tokio::net::{TcpListener};
use tokio::{io as tio};


pub struct DummyDevice {
    listener: TcpListener,
}

impl DummyDevice {
    pub fn new(addr: IpAddr) -> tio::Result<Self> {
        TcpListener::bind(&SocketAddr::new(addr, 0))
        .map(|listener| DummyDevice { listener })
    }

    pub fn address(&self) -> tio::Result<SocketAddr> {
        self.listener.local_addr()
    }

    pub fn run(self, conns: u64) -> impl Future<Item=(), Error=()> {
        self.listener.incoming()
        .map_err(|e| panic!(e))
        .take(conns)
        .for_each(|sock| {
            let (reader, writer) = sock.split();
            let reader = BufReader::new(reader);
            tokio::spawn(
                tio::read_until(reader, b'\n', Vec::new())
                .and_then(move |(reader, buf)| {
                    let response = if buf.starts_with(b"IDN?") {
                        &b"DummyDevice\n"[..]
                    } else {
                        &b"Error\n"[..]
                    };
                    tio::write_all(writer, response)
                    .map(|writer| {
                        //(reader, writer)
                        ()
                    })
                })
                .map_err(|e| panic!(e))
            )
        })
    }
}
