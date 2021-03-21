use std::{cell::Cell, unimplemented};
use std::io;

use crate::{prelude::Stream, stream::{Position, Transport}};

use super::Socket;


pub struct Pub {
    inner: Cell<Stream>,
}

impl Pub {
    pub fn send(&mut self, bytes: Vec<Vec<u8>>) -> io::Result<()> {
        let stream = self.inner.get_mut();
        let transport = stream.ensure_connected();

        // depending on the transport used we may or may not have to perform
        // filtering locally.
        match transport {
            Transport::Tcp(Position::Connect(_)) => {
                // we're on the connected end of a TCP session so rely on
                // the client in order to perform filtering.

                <Self as Socket>::send(self, bytes.iter())?;
            },

            _ => unimplemented!(),
        }

        Ok(())
    }
}

impl From<Stream> for Pub {
    fn from(inner: Stream) -> Self {
        Self {
            inner: Cell::new(inner),
        }
    }
}

impl Socket for Pub {
    fn stream(&mut self) -> &mut Stream {
        self.inner.get_mut()
    }
}
