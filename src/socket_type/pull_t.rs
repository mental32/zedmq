use std::io::{self, Read};

use crate::stream::Stream;

use super::Socket;

/// A zmq PULL socket.
#[derive(Debug)]
pub struct Pull {
    inner: Stream,
}

impl Pull {
    /// Block until a handshake has succeeded with `address`.
    pub fn connect(address: &str) -> io::Result<Self> {
        <Self as Socket>::connect(address)
    }

    /// Receive a multi-part message.
    pub fn recv(&mut self) -> io::Result<Vec<Vec<u8>>> {
        <Self as Socket>::recv(self)
    }
}

impl Socket for Pull {
    fn bind(_: &str) -> io::Result<Self> {
        unimplemented!()
    }

    fn connect(address: &str) -> io::Result<Self> {
        Ok(Self {
            inner: Stream::connected("PULL", address),
        })
    }

    fn transport(&mut self) -> &mut crate::stream::Transport {
        self.inner.transport.as_mut().unwrap()
    }
}
