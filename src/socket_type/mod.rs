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
    fn stream(&mut self) -> &mut Stream;

    /// Read bytes into some buffer.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream().read(buf)
    }

    /// Read bytes into some buffer.
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stream().write(buf)
    }

    #[inline]
    fn send<'a, I, S>(&mut self, mut data: I) -> io::Result<()>
    where
        I: DoubleEndedIterator<Item = &'a S>,
        S: AsRef<[u8]>,
        S: 'a,
    {
        let tail = data.next_back().expect("Can not send empty frame.");
        let body: Vec<_> = data.collect();

        let mut frame = Vec::with_capacity(max(
            body.iter().map(|i| i.as_ref().len()).max().unwrap_or(0),
            tail.as_ref().len(),
        ));

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

        // SHORT MESSAGE LAST
        frame.push(0x00);
        // SHORT SIZE
        frame.push(tail.as_ref().len() as u8);
        frame.extend_from_slice(&tail.as_ref());

        self.write(&frame)?;

        Ok(())
    }

    /// Read a frame and return a `FrameBuf` containing it.
    #[inline]
    fn recv_frame(&mut self) -> io::Result<FrameBuf> {
        let tag = {
            let mut tag = [0xFFu8];
            self.read(&mut tag)?;
            tag[0]
        };

        let (size, offset) = match tag {
            0x0 | 0x1 | 0x4 => {
                let mut tag = [0xFFu8];
                self.read(&mut tag)?;
                (tag[0] as usize, 2)
            }

            0x2 | 0x3 | 0x6 => {
                let mut head = [0; 8];
                self.read(&mut head)?;
                (u64::from_be_bytes(head) as usize, 9)
            }

            _ => unreachable!(),
        };

        let mut raw_frame = Vec::with_capacity(size + 2);

        raw_frame.push(tag);

        match offset {
            2 => raw_frame.push(size as u8),
            9 => raw_frame.extend_from_slice(&u64::to_be_bytes(size as u64)),
            _ => unreachable!(),
        }

        // dbg!((tag, size, &raw_frame, &raw_frame[offset..].len()));

        if size > 0 {
            let mut bytes = self.stream().bytes();

            for _ in 0..size {
                raw_frame.push(bytes.next().unwrap().unwrap())
            }
        }

        // dbg!((tag, size, &raw_frame));

        let frame_buf = FrameBuf::new(raw_frame);
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
