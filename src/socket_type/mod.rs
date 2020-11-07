use std::io;

use crate::codec::{Frame, FrameBuf};

pub mod pull_t;

// -- trait Socket

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
