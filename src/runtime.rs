#[cfg(feature = "runtime-async-std")]
#[allow(unused_imports)]
pub(crate) use {
    async_std::{
        io::Error,
        net::{TcpListener, TcpStream},
    },
    futures_util::io::{
        AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter,
    },
};

#[cfg(feature = "runtime-tokio")]
#[allow(unused_imports)]
pub(crate) use tokio::{
    io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter, Error},
    net::{TcpListener, TcpStream},
};
