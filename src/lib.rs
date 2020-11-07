//! > Note that this library is in the very early stages of development!
//! > Anything and everything may change!
//! 
//! # zedmq
//!
//! ## A Lightweight, Safe, pure-Rust ØMQ/ZMTP library implementation
//!
//! ## Index
//!
//! * [Brief](#brief)
//!
//! ### Brief
//!
//! _Zedmq_ is a native implementation of ØMQ in Rust focusing on:
//!
//! * being as lightweight as possible.
//! * being completely safe.
//! * providing a simple, blocking, obvious API.
//!
//! ## Examples
//!
//! ```rust
//! use zedmq::prelude::*;
//!
//! fn main() -> std::io::Result<()> {
//!     let mut socket = <Pull as Socket>::bind("tcp://127.0.0.1:5678")?;
//!
//!     while Some(message) = socket.recv() {
//!         dbg!(message);
//!     }
//!
//!     Ok(())
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::{
    io,
    io::{Read, Write},
    net::TcpStream,
};

mod codec;

use codec::{Frame, FrameBuf, FrameKind};

/// The prelude.
pub mod prelude {
    pub use crate::{Pull, Socket};
}

fn read_stream<R>(stream: &mut R, buf: &mut [u8]) -> io::Result<usize>
where
    R: Read,
{
    let n = stream.read(buf)?;
    if n == 0 {
        Err(io::Error::new(
            io::ErrorKind::WriteZero,
            "No bytes were written.",
        ))
    } else {
        Ok(n)
    }
}

/// A trait used to generalize ZMQ behaviour.
pub trait Socket
where
    Self: Sized,
{
    /// Bind to some address.
    fn bind(address: &str) -> io::Result<Self>;

    /// Connect to some address.
    fn connect(address: &str) -> io::Result<Self>;

    /// Receive a frame into an owned vec.
    fn recv_frame<'a>(&mut self) -> io::Result<FrameBuf>;

    /// Receive a frame into a pre-allocated slice-buffer.
    fn recv_frame_into<'a>(&mut self, buf: &'a mut [u8]) -> io::Result<Frame<'a>>;

    /// Receive a multi-part message.
    fn recv(&mut self) -> io::Result<Vec<Vec<u8>>>;
}

/// A zmq PULL socket.
#[derive(Debug)]
pub struct Pull {
    stream: TcpStream,
}

/// # Examples
/// ```
/// use zedmq::codec::greeting;
///
/// let g = greeting();
/// let (left, right) = (&g[..=11], &g[12..]);
/// assert_eq!(left, &[0xFFu8, 0, 0, 0, 0, 0, 0, 0, 0, 0x7F, 3, 0] as &[u8]);
/// assert_eq!(&right[..4], b"NULL" as &[u8]);
/// ```
fn greeting() -> [u8; 64] {
    // TODO: Missing a way to specify the security mechanism (currently NULL) and the as-server field (currently false)

    let mut raw = [0u8; 64];

    raw[0] = 0xFF; // signature start
                   // signature padding.
    raw[9] = 0x7F; // signature end
    raw[10] = 3;
    raw[11] = 0;

    // Security
    raw[12] = 0x4E;
    raw[13] = 0x55;
    raw[14] = 0x4C;
    raw[15] = 0x4C;

    raw
}

impl Socket for Pull {
    fn recv(&mut self) -> io::Result<Vec<Vec<u8>>> {
        let mut frames = vec![];

        loop {
            let frame_buf = self.recv_frame()?;
            assert!(frame_buf.as_frame().kind().is_some());

            if let Some(message) = frame_buf.as_frame().try_into_message() {
                frames.push(message.body().to_vec());

                if message.is_last() {
                    break;
                }
            } else {
                assert!(matches!(
                    frame_buf.as_frame().kind(),
                    Some(FrameKind::Command)
                ));

                panic!(
                    "Unexpected command frame! {:#?}",
                    frame_buf.as_frame().try_into_command()
                );
            }
        }

        Ok(frames)
    }

    fn recv_frame_into<'a>(&mut self, buf: &'a mut [u8]) -> io::Result<Frame<'a>> {
        let n = read_stream(&mut self.stream, buf)?;
        let byte_slice = &buf[..n];
        let frame_slice = Frame::new(byte_slice);
        Ok(frame_slice)
    }

    fn recv_frame<'a>(&mut self) -> io::Result<FrameBuf> {
        // FLAGS + (one or eight octets)
        let mut head = [0; 9];

        let head_n = read_stream(&mut self.stream, &mut head)?;

        let phantom_frame = Frame::new(&head as &[_]);
        let size = phantom_frame.size().unwrap();

        if size <= head.len() {
            Ok(FrameBuf::new(head[..head_n].to_vec()))
        } else {
            let mut tail = Vec::<u8>::with_capacity(size - head.len());
            let tail_n = read_stream(&mut self.stream, tail.as_mut_slice())?;

            let mut data = head[..head_n].to_vec();
            data.extend_from_slice(&tail.as_slice()[..tail_n]);

            Ok(FrameBuf::new(data))
        }
    }

    fn bind(_: &str) -> io::Result<Self> {
        unimplemented!()
    }

    fn connect(address: &str) -> io::Result<Self> {
        let mut stream = TcpStream::connect(address)?;

        let greeting = greeting();
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
            let n = stream.read(&mut buf[..])?;

            dbg!(&buf[..n]);

            let handshake = Frame::from(&buf[..n]);

            dbg!(&handshake.try_into_command());
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

        Ok(Self { stream })
    }
}
