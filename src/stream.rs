use std::{
    io::{self, Read, Write},
    net::TcpListener,
    net::TcpStream,
};

use crate::codec::{FrameBuf, ZMTP};

// -- Transport

#[derive(Debug)]
pub(crate) enum Position<L, R> {
    Connect(L),
    Bind(R),
}

#[derive(Debug)]
pub(crate) enum Transport {
    Tcp(Position<TcpStream, TcpListener>),
}

impl Write for Transport {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Self::Tcp(Position::Connect(stream)) => stream.write(buf),
            Self::Tcp(Position::Bind(_)) => unimplemented!(),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::Tcp(Position::Connect(stream)) => stream.flush(),
            Self::Tcp(Position::Bind(_)) => Ok(()),
        }
    }
}

impl Read for Transport {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Tcp(Position::Connect(stream)) => stream.read(buf),
            Self::Tcp(Position::Bind(_)) => unimplemented!(),
        }
    }
}

// -- Stream<'a>

/// The stream struct represents the underlying connection primitive.
#[derive(Debug, Default)]
pub(crate) struct Stream {
    socket_type: &'static str,
    address: String,
    pub(crate) transport: Option<Transport>,
}


impl Stream {
    /// Given an `address` produce a `Stream` that is connected even if connecting may block.
    pub(super) fn connected(socket_type: &'static str, address: &str) -> Self {
        let mut stream = Self {
            socket_type,
            address: address.to_string(),
            transport: None,
        };

        stream.ensure_connected();

        stream
    }

    pub(super) fn connect(&self) -> io::Result<Transport> {
        let address = self.address.clone();
        let produce = move || Ok(Transport::Tcp(Position::Connect(TcpStream::connect(address)?)));

        let transport = ZMTP::connect(produce)?
            .greet(crate::ZMQ_VERSION, false)?
            .ready(self.socket_type)?;

        Ok(transport)
    }

    fn ensure_connected(&mut self) -> &mut Transport {
        while self.transport.is_none() {
            if let Ok(fresh) = self.connect() {
                let _ = self.transport.replace(fresh);
                break;
            } else {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }

        self.transport.as_mut().unwrap()
    }
}

impl Read for Stream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut n_bytes;

        while {
            n_bytes = self.ensure_connected().read(buf)?;
            n_bytes
        } <= 0
        {
            self.transport.take();
        }

        Ok(n_bytes)
    }
}
