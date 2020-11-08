use std::{
    io::Write,
    io::{self, Read},
    net::TcpStream,
};

use crate::codec::FrameBuf;

// -- Stream<'a>

#[derive(Debug, Default)]
pub(crate) struct Stream {
    address: String,
    inner: Option<TcpStream>,
}

impl From<(String, TcpStream)> for Stream {
    fn from((address, inner): (String, TcpStream)) -> Self {
        Self {
            address,
            inner: Some(inner),
        }
    }
}

impl Stream {
    pub(super) fn connected(address: &str) -> Self {
        let mut stream = Self {
            address: address.to_string(),
            inner: None,
        };

        stream.ensure_connected();

        stream
    }

    pub(super) fn connect(address: &str) -> io::Result<TcpStream> {
        let mut stream = TcpStream::connect(address)?;

        // let greeting = crate::codec::Greeting::build().as_server(true).as_bytes();
        let mut greeting = crate::codec::greeting();
        let (partial, remaining) = (&greeting[..=11], &greeting[12..]);

        // Send partial greeting
        stream.write(partial)?;

        // Inspect remote partial greeting.
        {
            let mut buf = [0u8; 12];
            let n = stream.read(&mut buf[..])?;
            assert_eq!(n, 12, "{:?}", buf);
            // TODO: parse partial greeting for correct peer zmq version.
        }

        // Send remaining greeting
        stream.write(remaining)?;

        {
            // Read the remaining remote greeting.
            let mut buf = [0u8; 52];
            let n = stream.read(&mut buf[..])?;
            assert_eq!(n, 52);
            // TODO: parse, this contains the security mechanism (by default NULL) and some extra metadata.

            // Inspect remote handshake.
            let mut buf = [0u8; 64];
            let _n = stream.read(&mut buf[..])?;

            // dbg!(&buf[..n]);

            // let handshake = Frame::from(&buf[..n]);

            // dbg!(&handshake.try_into_command());
            // TODO: validate handshake, this contains (for NULL security mechanisms) the following properties:
            //  - Socket-Type {type} i.e. PUSH, PULL, DEALER, ROUTER, PAIR
            //  - Identity; only if WE are ROUTER and they are using a ROUTER compatible socket type with a custom routing id.
        }

        // Send handshake

        let handshake = {
            let properties = vec![("Socket-Type", "PULL")];

            FrameBuf::short_command("READY", Some(properties))
        };

        stream.write(handshake.as_ref())?;

        Ok(stream)
    }

    fn ensure_connected(&mut self) -> &mut TcpStream {
        while self.inner.is_none() {
            match Self::connect(self.address.as_str()) {
                Ok(stream) => {
                    let _ = self.inner.replace(stream);
                    break;
                }

                _ => std::thread::sleep(std::time::Duration::from_millis(100)),
            }
        }

        self.inner.as_mut().unwrap()
    }

    pub(super) fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            let n_bytes = self.ensure_connected().read(buf)?;

            if n_bytes > 0 {
                return Ok(n_bytes);
            }

            self.inner.take();
        }
    }
}
