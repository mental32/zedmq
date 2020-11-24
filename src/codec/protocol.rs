use std::io::{self, Read, Write};

use crate::stream::{Position, Transport};

use super::FrameBuf;

/// A builder struct used to handle `greeting` and `handshake` steps.
pub(crate) struct ZMTP {
    security: Option<()>,
    transport: Transport,
}

impl ZMTP {
    /// The "connect" side of the connection.
    pub fn connect<F>(produce_transport: F) -> io::Result<Self>
    where
        F: FnOnce() -> io::Result<Transport>,
    {
        let transport = produce_transport()?;

        Ok(Self {
            security: None,
            transport,
        })
    }

    /// Perform the greeting step of the ZMTP spec.
    pub fn greet(
        mut self,
        (major, minor, _): (u8, u8, u8),
        as_server: bool,
    ) -> io::Result<Self> {
        let (partial, remaining) = {
            let mut greeting = crate::codec::Greeting::build();
            greeting.as_server(as_server);
            // greeting.security(self.security)
            greeting.into_parts()
        };

        // Send partial greeting
        self.transport.write(&partial)?;

        // unreachable!();

        // Inspect remote partial greeting.
        {
            let mut buf = [0u8; 12];
            let n = self.transport.read(&mut buf)?;
            assert_eq!(n, 12, "{:?}", buf);

            // let peer_major = buf[10];

            // if peer_major != major {
            //     return Err(io::Error::new(io::ErrorKind::ConnectionAborted, "peer major is not the same as us."))
            // }

            // let peer_minor = buf[11];

            // if peer_minor > minor {
            //     return Err(io::Error::new(io::ErrorKind::ConnectionAborted, "peer minor is higher than us."))
            // }
        }

        // Send remaining greeting
        self.transport.write(&remaining)?;

        // unreachable!();

        Ok(self)
    }

    pub fn ready<'b>(self, socket_type: &'b str) -> io::Result<Transport> {
        let Self { mut transport, .. } = self;

        // unreachable!();

        {
            // Read the remaining remote greeting.
            let mut buf = [0u8; 52];
            let n = transport.read(&mut buf[..])?;
            assert_eq!(n, 52);
            // TODO: parse, this contains the security mechanism (by default NULL) and some extra metadata.

            // Inspect remote handshake.
            let mut buf = [0u8; 64];
            let n = transport.read(&mut buf)?;

            // for octet in &buf {

            // }

            dbg!(super::Frame { bytes: &buf }.try_into_command());

            // TODO: validate handshake, this contains (for NULL security mechanisms) the following properties:
            //  - Socket-Type {type} i.e. PUSH, PULL, DEALER, ROUTER, PAIR
            //  - Identity; only if WE are ROUTER and they are using a ROUTER compatible socket type with a custom routing id.
        }

        // Send handshake

        let handshake = {
            let properties = vec![("Socket-Type", socket_type)];

            FrameBuf::short_command("READY", Some(properties))
        };

        dbg!(handshake.as_frame().try_into_command());

        transport.write(handshake.as_ref())?;

        // unreachable!();

        Ok(transport)
    }
}
