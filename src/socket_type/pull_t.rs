use std::io::{self};

use crate::stream::Stream;

use super::Socket;

/// A zmq PULL socket.
#[derive(Debug)]
pub struct Pull {
    inner: Stream,
}

impl From<Stream> for Pull {
    fn from(inner: Stream) -> Self {
        Self { inner }
    }
}

impl Pull {
    /// Receive a multi-part message.
    pub fn recv(&mut self) -> io::Result<Vec<Vec<u8>>> {
        <Self as Socket>::recv(self)
    }
}

impl Socket for Pull {
    fn stream(&mut self) -> &mut crate::stream::Stream {
        &mut self.inner
    }
}
