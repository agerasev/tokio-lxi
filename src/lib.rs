extern crate tokio;

#[cfg(test)]
mod dummy;

use std::io::{BufReader};
use std::net::{SocketAddr};

use tokio::prelude::*;
use tokio::net::{TcpStream};
use tokio::{io as tio};


fn remove_newline(text: &mut String) {
    match text.pop() {
        Some('\n') => match text.pop() {
            Some('\r') => (),
            Some(c) => text.push(c),
            None => (),
        },
        Some(c) => text.push(c),
        None => (),
    }
}

pub struct LxiDevice {
    inp: Option<BufReader<tio::ReadHalf<TcpStream>>>,
    out: Option<tio::WriteHalf<TcpStream>>,
}

impl LxiDevice {
    pub fn connect(addr: &SocketAddr)
    -> impl Future<Item=LxiDevice, Error=tio::Error> {
        TcpStream::connect(&addr)
        .map(|stream| {
            let (inp, out) = stream.split();
            LxiDevice {
                inp: Some(BufReader::new(inp)),
                out: Some(out),
            }
        })
    }

    pub fn write<T: AsRef<[u8]>>(mut self, buf: T)
    -> impl Future<Item=(LxiDevice, T), Error=tio::Error> {
        tio::write_all(self.out.take().unwrap(), buf)
        .map(|(out, buf)| {
            assert!(self.out.is_none());
            self.out = Some(out);
            (self, buf)
        })
    }

    pub fn read<T: AsMut<[u8]>>(mut self, buf: T)
    -> impl Future<Item=(LxiDevice, T, usize), Error=tio::Error> {
        tio::read(self.inp.take().unwrap(), buf)
        .map(|(inp, buf, count)| {
            assert!(self.inp.is_none());
            self.inp = Some(inp);
            (self, buf, count)
        })
    }

    pub fn read_until(mut self, byte: u8, buf: Vec<u8>)
    -> impl Future<Item=(LxiDevice, Vec<u8>), Error=tio::Error> {
        tio::read_until(self.inp.take().unwrap(), byte, buf)
        .map(|(inp, buf)| {
            assert!(self.inp.is_none());
            self.inp = Some(inp);
            (self, buf)
        })
    }

    pub fn send(self, text: String)
    -> impl Future<Item=(LxiDevice, String), Error=tio::Error> {
        self.write(text)
        .and_then(|(dev, text)| {
            dev.write(b"\r\n")
            .map(|(dev, _)| {
                (dev, text)
            })
        })
    }

    pub fn receive(self, text: String)
    -> impl Future<Item=(LxiDevice, String), Error=tio::Error> {
        self.read_until(b'\n', text.into_bytes())
        .then(|result| {
            match result {
                Ok((dev, buf)) => {
                    match String::from_utf8(buf) {
                        Ok(mut text) => {
                            remove_newline(&mut text);
                            Ok((dev, text))
                        },
                        Err(_) => Err(tio::Error::new(
                            tio::ErrorKind::InvalidData,
                            ""
                        )),
                    }
                },
                Err(err) => Err(err),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::net::{IpAddr, Ipv4Addr};
    
    use tokio::net::{TcpListener};

    use dummy::DummyEmulator;

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
    fn dummy_idn_stream() {
        let device = DummyEmulator::new(LOCALHOST).unwrap();
        let address = device.address().unwrap();
        let server = device.run(1);

        let client = TcpStream::connect(&address)
        .and_then(|stream| {
            tio::write_all(stream, b"IDN?\r\n")
        })
        .and_then(|(stream, _text)| {
            let (reader, _writer) = stream.split();
            let reader = BufReader::new(reader);
            tio::read_until(reader, b'\n', Vec::new())
        })
        .map(|(_, buf)| {
            assert_eq!(b"DummyEmulator\r\n", &buf[..]);
        })
        .map_err(|e| panic!(e));

        tokio::run(server.join(client).map(|_| ()));
    }

    #[test]
    fn dummy_idn_device() {
        let device = DummyEmulator::new(LOCALHOST).unwrap();
        let address = device.address().unwrap();
        let server = device.run(1);

        let device = LxiDevice::connect(&address)
        .and_then(|dev| {
            dev.send(String::from("IDN?"))
        })
        .and_then(|(dev, _)| {
            dev.receive(String::new())
        })
        .map(|(_, text)| {
            assert_eq!("DummyEmulator", text);
        })
        .map_err(|e| panic!(e));

        tokio::runtime::current_thread::run(server.join(device).map(|_| ()));
    }
}
