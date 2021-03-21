use std::{
    cmp::max,
    io::{self, Read, Write},
};

use crate::{
    codec::{Frame, FrameBuf, FrameKind},
    stream::Stream,
};

pub mod pull_t;
pub mod push_t;
pub mod rep_t;
pub mod req_t;
pub mod sub_t;

// -- LazyMessage

/// A lazy message will iterate over frames of a message until it hits a tail at which point it fuses.
pub(crate) struct LazyMessage<'a> {
    stream: &'a mut Stream,
    witness: bool,
}

impl<'a> From<&'a mut Stream> for LazyMessage<'a> {
    fn from(stream: &'a mut Stream) -> Self {
        Self {
            stream,
            witness: false,
        }
    }
}

impl Iterator for LazyMessage<'_> {
    type Item = io::Result<FrameBuf>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.witness {
            return None;
        }

        let frame = self.stream.recv_frame();

        if let Ok(ref frame) = frame {
            match frame.as_frame().kind()? {
                FrameKind::MessageTail => self.witness = true,
                FrameKind::MessagePart => (),
                _ => return None,
            }
        }

        Some(frame)
    }
}

// -- trait Socket

/// A trait used to generalize ZMQ behaviour.
///
/// This trait doesn't care about the underlying socket type so
/// acceptable behaviour is left up to the trait implementors like `Pull`
///
pub trait Socket
where
    Self: Sized,
{
    /// Get a mutable reference to the current transport primitive.
    fn stream(&mut self) -> &mut Stream;

    /// Read bytes into some buffer.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream().ensure_connected().read(buf)
    }

    /// Read bytes into some buffer.
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stream().ensure_connected().write(buf)
    }

    #[inline]
    fn send<'a, I, S>(&mut self, mut data: I) -> io::Result<()>
    where
        I: DoubleEndedIterator<Item = &'a S>,
        S: AsRef<[u8]>,
        S: 'a,
    {
        let tail = data
            .next_back()
            .ok_or(io::Error::from(io::ErrorKind::InvalidInput))?;

        let body: Vec<_> = data.collect();

        let capacity = max(
            body.iter().map(|i| i.as_ref().len()).max().unwrap_or(0),
            tail.as_ref().len(),
        );

        let mut frame = Vec::with_capacity(capacity);

        for part in body {
            let size = part.as_ref().len();

            if size <= std::u8::MAX as usize {
                // SHORT MESSAGE MORE
                frame.push(0x01);
                // SHORT SIZE
                frame.push(size as u8);
            } else {
                // LONG MESSAGE MORE
                frame.push(0x03);
                // SHORT SIZE
                frame.extend_from_slice(&(size as u32).to_be_bytes() as &[_]);
            };

            frame.extend_from_slice(part.as_ref());

            self.write(&frame)?;

            frame.clear();
        }

        frame.clear();

        // SHORT MESSAGE LAST
        frame.push(0x00);
        // SHORT SIZE
        frame.push(tail.as_ref().len() as u8);
        frame.extend_from_slice(&tail.as_ref());

        self.write(&frame)?;

        Ok(())
    }

    /// Read a frame and write it into the provided buffer slice.
    fn recv_frame_into<'b>(&mut self, buf: &'b mut [u8]) -> io::Result<Frame<'b>> {
        let n = self.read(buf)?;
        let byte_slice = &buf[..n];
        let frame_slice = Frame::new(byte_slice);

        Ok(frame_slice)
    }

    // Receive a multi-part message as a 2d vec of bytes.
    #[inline]
    fn recv(&mut self) -> io::Result<Vec<Vec<u8>>> {
        let mut frames = vec![];

        loop {
            let frame_buf = self.stream().recv_frame()?;
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
