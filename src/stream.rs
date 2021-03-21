use std::io::{self, Read, Write};
use std::net::TcpListener;
use std::net::TcpStream;

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
pub struct Stream {
    socket_type: &'static str,
    address: String,
    transport: Option<Transport>,
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
        let produce = move || {
            Ok(Transport::Tcp(Position::Connect(TcpStream::connect(
                address,
            )?)))
        };

        let transport = ZMTP::connect(produce)?
            .greet(crate::ZMQ_VERSION, false)?
            .ready(self.socket_type)?;

        Ok(transport)
    }

    pub(crate) fn ensure_connected(&mut self) -> &mut Transport {
        while self.transport.is_none() {
            if let Ok(fresh) = self.connect() {
                let _ = self.transport.replace(fresh);
                break;
            } else {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }

        match self.transport.as_mut() {
            Some(inner) => inner,
            None => unreachable!(),
        }
    }

    /// Read a frame and return a `FrameBuf` containing it.
    #[inline]
    pub(crate) fn recv_frame(&mut self) -> io::Result<FrameBuf> {
        let tag = {
            let mut tag = [0xFFu8];
            self.read(&mut tag)?;
            tag[0]
        };

        let (size, offset) = match tag {
            0x0 | 0x1 | 0x4 => {
                let mut tag = [0xFFu8];
                self.read(&mut tag)?;
                (tag[0] as usize, 2)
            }

            0x2 | 0x3 | 0x6 => {
                let mut head = [0; 8];
                self.read(&mut head)?;
                (u64::from_be_bytes(head) as usize, 9)
            }

            _ => return Err(io::Error::from(io::ErrorKind::InvalidData)),
        };

        let mut raw_frame = Vec::with_capacity(size + 2);

        raw_frame.push(tag);

        match offset {
            2 => raw_frame.push(size as u8),
            9 => raw_frame.extend_from_slice(&u64::to_be_bytes(size as u64)),
            _ => unreachable!(),
        }

        if size > 0 {
            let mut bytes = self.bytes();

            for _ in 0..size {
                let byte = bytes.next().ok_or(io::Error::from(io::ErrorKind::UnexpectedEof))??;
                raw_frame.push(byte)
            }
        }

        let frame_buf = FrameBuf::new(raw_frame);
        Ok(frame_buf)
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

impl Write for Stream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.ensure_connected().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.ensure_connected().flush()
    }
}
