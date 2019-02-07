extern crate tokio;

#[cfg(test)]
mod dummy;

use std::net::{SocketAddr, IpAddr, Ipv4Addr};




#[cfg(test)]
mod tests {
    use super::*;

    use std::io::{BufReader};
    
    use tokio::prelude::*;
    use tokio::net::{TcpListener, TcpStream};
    use tokio::{io as tio};

    use dummy::DummyDevice;

    pub static LOCALHOST: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);

    #[test]
    fn client_server() {
        let listener = TcpListener::bind(&SocketAddr::new(LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let server = listener.incoming()
        .map_err(|e| panic!(e))
        .take(1)
        .for_each(|sock| {
            let (reader, writer) = sock.split();
            tokio::spawn(
                tio::copy(reader, writer)
                .map(|_| ())
                .map_err(|e| panic!(e))
            )
        });

        let client = TcpStream::connect(&address)
        .and_then(|stream| {
            tio::write_all(stream, b"hello, server\n")
        })
        .and_then(|(stream, text)| {
            let buf = vec!(0; text.len());
            tio::read_exact(stream, buf)
            .map(move |(stream, buf)| (stream, text, buf))
        })
        .map(|(_, text, buf)| {
            assert_eq!(text, &buf[..]);
        })
        .map_err(|e| panic!(e));

        tokio::run(server.join(client).map(|_| ()));
    }

    #[test]
    fn dummy_idn() {
        let device = DummyDevice::new(LOCALHOST).unwrap();
        let address = device.address().unwrap();
        let server = device.run(1);

        let client = TcpStream::connect(&address)
        .and_then(|stream| {
            tio::write_all(stream, b"IDN?\r\n")
        })
        .and_then(|(stream, text)| {
            let (reader, writer) = stream.split();
            let reader = BufReader::new(reader);
            tio::read_until(reader, b'\n', Vec::new())
        })
        .map(|(_, buf)| {
            assert_eq!(b"DummyDevice\n", &buf[..]);
        })
        .map_err(|e| panic!(e));

        tokio::run(server.join(client).map(|_| ()));
    }
}
