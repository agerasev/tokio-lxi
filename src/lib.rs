#[cfg(test)]
mod dummy;
mod runtime;

use runtime::{AsyncBufRead, AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter, TcpStream};
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] runtime::Error),

    #[error("Invalid response data (lossy decoding from UTF-8): {0}")]
    ResponseDataInvalid(String),

    #[error(transparent)]
    UserCallbackError(#[from] anyhow::Error),
}

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
    stream: Pin<Box<BufReader<BufWriter<TcpStream>>>>,
}

impl LxiDevice {
    pub async fn connect(addr: &SocketAddr) -> Result<Self, Error> {
        Self::connect_with_buffer_capacity(addr, 1024, 128).await
    }

    pub async fn connect_with_buffer_capacity(
        addr: &SocketAddr,
        read_buffer_size: usize,
        write_buffer_size: usize,
    ) -> Result<Self, Error> {
        let stream = BufReader::with_capacity(
            read_buffer_size,
            BufWriter::with_capacity(write_buffer_size, TcpStream::connect(&addr).await?),
        );
        Ok(Self {
            stream: Box::pin(stream),
        })
    }

    async fn write<T: AsRef<[u8]>>(&mut self, buf: T) -> Result<(), Error> {
        self.stream.write_all(buf.as_ref()).await?;
        Ok(())
    }

    pub async fn send(&mut self, req: &str) -> Result<(), Error> {
        self.write(req).await?;
        self.stream.write_all(b"\r\n").await?;
        self.stream.flush().await?;
        Ok(())
    }

    pub async fn receive(&mut self) -> Result<String, Error> {
        let mut buf = vec![];
        self.stream.read_until(b'\n', &mut buf).await?;
        let mut response = String::from_utf8(buf).map_err(|error| {
            Error::ResponseDataInvalid(String::from_utf8_lossy(error.as_bytes()).into_owned())
        })?;

        remove_newline(&mut response);

        Ok(response)
    }

    pub async fn receive_data<'a, T, F, P>(&'a mut self, parser: P) -> Result<T, Error>
    where
        F: Future<Output = Result<T, Error>> + Send,
        P: FnOnce(Pin<&'a mut (dyn AsyncBufRead + Send)>) -> F,
    {
        let stream = self.stream.as_mut();
        Ok(parser(stream).await?)
    }

    pub async fn request(&mut self, req: &str) -> Result<String, Error> {
        self.send(req).await?;
        self.receive().await
    }

    pub async fn request_data<'a, T, F, P>(&'a mut self, req: &str, parser: P) -> Result<T, Error>
    where
        F: Future<Output = Result<T, Error>> + Send,
        P: FnOnce(Pin<&'a mut (dyn AsyncBufRead + Send)>) -> F,
    {
        self.send(req).await?;
        self.receive_data(parser).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use dummy::DummyEmulator;
    use runtime::{AsyncReadExt, BufReader, TcpListener};
    use std::net::{IpAddr, Ipv4Addr};

    pub static LOCALHOST: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);

    #[tokio::test]
    async fn client_server() {
        let mut server: TcpListener = TcpListener::bind(&SocketAddr::new(LOCALHOST, 0))
            .await
            .unwrap();
        let address = server.local_addr().unwrap();

        let server_future = async move {
            let (mut conn, _): (TcpStream, _) = server.accept().await.unwrap();
            let (mut reader, mut writer) = conn.split();
            tokio::io::copy(&mut reader, &mut writer).await.unwrap();
        };

        let client_future = async move {
            let mut client: TcpStream = TcpStream::connect(&address).await.unwrap();
            let (mut reader, mut writer) = client.split();
            let request = b"hello, server\n";
            writer.write_all(request).await.unwrap();
            let mut reply = vec![0; request.len()];
            reader.read_exact(&mut reply).await.unwrap();

            assert_eq!(&request[..], &reply[..]);
        };

        let (_, _) = tokio::join!(server_future, client_future);
    }

    #[tokio::test]
    async fn dummy_idn_stream() {
        let mut device = DummyEmulator::start(LOCALHOST).await;
        let address = device.address().unwrap();
        let server_future = device.run(1);

        let client_future = async move {
            let mut client: TcpStream = TcpStream::connect(&address).await.unwrap();
            let (reader, mut writer) = client.split();
            writer.write_all(b"IDN?\r\n").await.unwrap();
            let mut reader = BufReader::new(reader);
            let mut reply = vec![];
            reader.read_until(b'\n', &mut reply).await.unwrap();

            assert_eq!(&b"DummyEmulator\r\n"[..], &reply[..]);
        };

        tokio::join!(server_future, client_future);
    }

    #[tokio::test]
    async fn dummy_idn_device() {
        let mut device = DummyEmulator::start(LOCALHOST).await;
        let address = device.address().unwrap();
        let server_future = device.run(1);

        let client_future = async move {
            let mut device = LxiDevice::connect(&address).await.unwrap();
            device.send("IDN?").await.unwrap();
            let response = device.receive().await.unwrap();
            assert_eq!("DummyEmulator", response);
        };

        tokio::join!(server_future, client_future);
    }
}
