use std::io::{self};

use crate::stream::Stream;

use super::Socket;

/// A zmq PUSH socket.
#[derive(Debug)]
pub struct Push {
    inner: Stream,
}

impl From<Stream> for Push {
    fn from(inner: Stream) -> Self {
        Self { inner }
    }
}

impl Push {
    /// Send a message.
    pub fn send(&mut self, bytes: Vec<Vec<u8>>) -> io::Result<()> {
        <Self as Socket>::send(self, bytes.iter())
    }
}

impl Socket for Push {
    fn stream(&mut self) -> &mut crate::stream::Stream {
        &mut self.inner
    }
}
