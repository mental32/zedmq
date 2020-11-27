use std::io::{self};

use crate::stream::Stream;

use super::Socket;

/// A zmq PUSH socket.
#[derive(Debug)]
pub struct Push {
    inner: Stream,
}

impl Push {
    /// Block until a handshake has succeeded with `address`.
    pub fn connect(address: &str) -> io::Result<Self> {
        <Self as Socket>::connect(address)
    }

    /// Send a message.
    pub fn send(&mut self, bytes: Vec<Vec<u8>>) -> io::Result<()> {
        <Self as Socket>::send(self, bytes.iter())
    }
}

impl Socket for Push {
    fn bind(_: &str) -> io::Result<Self> {
        unimplemented!()
    }

    fn connect(address: &str) -> io::Result<Self> {
        Ok(Self {
            inner: Stream::connected("PUSH", address),
        })
    }

    fn stream(&mut self) -> &mut crate::stream::Stream {
        &mut self.inner
    }
}
