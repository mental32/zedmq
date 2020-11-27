use std::io::{self};

use crate::stream::Stream;

use super::Socket;

// -- ReqPending

/// A zmq REQ socket pending a response.
#[derive(Debug)]
pub struct RepPending {
    inner: Stream,
}

impl RepPending {
    /// Send a message.
    pub fn send(mut self, bytes: Vec<Vec<u8>>) -> io::Result<Rep> {
        <Self as Socket>::send(&mut self, bytes.iter())?;
        Ok(Rep { inner: self.inner })
    }
}

impl Socket for RepPending {
    fn stream(&mut self) -> &mut crate::stream::Stream {
        &mut self.inner
    }
}

// -- Rep


impl From<Stream> for Rep {
    fn from(inner: Stream) -> Self {
        Self { inner }
    }
}
/// A zmq REP socket.
#[derive(Debug)]
pub struct Rep {
    inner: Stream,
}

impl Rep {
    /// Recieve a multipart message with the pending REP socket.
    pub fn recv(mut self) -> io::Result<(Vec<Vec<u8>>, RepPending)> {
        let data = <Self as Socket>::recv(&mut self)?;
        let Self { inner } = self;
        Ok((data, RepPending { inner }))
    }
}

impl Socket for Rep {
    fn stream(&mut self) -> &mut crate::stream::Stream {
        &mut self.inner
    }
}
