use std::{cmp::max, io::{self, Read, Write}};

use crate::{stream::Transport, codec::{Frame, FrameBuf, FrameKind}};

pub mod pull_t;

// -- trait Socket

/// A trait used to generalize ZMQ behaviour.
///
/// This trait doesn't care about the underlying socket type so
/// acceptable behaviour is left up to the trait implementors like `Pull`
pub(crate) trait Socket
where
    Self: Sized,
{
    /// Bind to some address.
    fn bind(address: &str) -> io::Result<Self>;

    /// Connect to some address.
    fn connect(address: &str) -> io::Result<Self>;

    /// Get a mutable reference to the current transport primitive.
    fn transport(&mut self) -> &mut Transport;

    /// Read bytes into some buffer.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>;

    #[inline]
    fn send(&mut self, data: &[u8]) -> io::Result<()> {
        Ok(())
    }

    /// Read a frame and return a `FrameBuf` containing it.
    #[inline]
    fn recv_frame(&mut self) -> io::Result<FrameBuf> {
        // FLAGS + (one or eight octets)
        let mut head = [0; 9];

        let head_n = self.read(&mut head)?;

        let phantom_frame = Frame::new(&head as &[_]);
        let size = phantom_frame.size().unwrap();

        let frame_buf = if size > head.len() {
            let mut tail = Vec::with_capacity(size - head.len());
            let tail_n = self.read(tail.as_mut_slice())?;

            let mut data = head[..head_n].to_vec();
            data.extend_from_slice(&tail.as_slice()[..tail_n]);

            FrameBuf::new(data)
        } else {
            FrameBuf::new(head[..head_n].to_vec())
        };

        Ok(frame_buf)
    }

    /// Read a frame and write it into the provided buffer slice.
    fn recv_frame_into<'b>(&mut self, buf: &'b mut [u8]) -> io::Result<Frame<'b>> {
        let n = self.read(buf)?;
        let byte_slice = &buf[..n];
        let frame_slice = Frame::new(byte_slice);

        Ok(frame_slice)
    }

    // Receive a multi-part message.
    #[inline]
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
}
