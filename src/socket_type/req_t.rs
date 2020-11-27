use std::io;

use crate::stream::Stream;

use super::Socket;

// -- ReqPending

/// A zmq REQ socket pending a response.
#[derive(Debug)]
pub struct ReqPending {
    inner: Stream,
}

impl ReqPending {
    pub fn recv(mut self) -> io::Result<(Vec<Vec<u8>>, Req)> {
        let data = <Self as Socket>::recv(&mut self)?;
        let Self { inner } = self;
        Ok((data, Req { inner }))
    }
}

impl Socket for ReqPending {
    fn bind(_: &str) -> io::Result<Self> {
        unimplemented!()
    }

    fn connect(address: &str) -> io::Result<Self> {
        Ok(Self {
            inner: Stream::connected("REQ", address),
        })
    }

    fn stream(&mut self) -> &mut crate::stream::Stream {
        &mut self.inner
    }
}

// -- Req

/// A zmq REQ socket.
#[derive(Debug)]
pub struct Req {
    inner: Stream,
}

impl Req {
    /// Block until a handshake has succeeded with `address`.
    pub fn connect(address: &str) -> io::Result<Self> {
        <Self as Socket>::connect(address)
    }

    /// Send a message.
    pub fn send(mut self, bytes: Vec<Vec<u8>>) -> io::Result<ReqPending> {
        <Self as Socket>::send(&mut self, bytes.iter())?;
        Ok(ReqPending { inner: self.inner })
    }
}

impl Socket for Req {
    fn bind(_: &str) -> io::Result<Self> {
        unimplemented!()
    }

    fn connect(address: &str) -> io::Result<Self> {
        Ok(Self {
            inner: Stream::connected("REQ", address),
        })
    }

    fn stream(&mut self) -> &mut crate::stream::Stream {
        &mut self.inner
    }
}
