use std::io;

use crate::codec::{Frame, FrameBuf, FrameKind};

use super::Socket;

/// A zmq PULL socket.
#[derive(Debug)]
pub struct Pull {
    inner: crate::stream::Stream,
}

impl Pull {
    /// Block until a handshake has succeeded with `address`.
    pub fn connect(address: &str) -> io::Result<Self> {
        <Self as Socket>::connect(address)
    }

    /// Receive a multi-part message.
    pub fn recv(&mut self) -> io::Result<Vec<Vec<u8>>> {
        <Self as Socket>::recv(self)
    }
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

    fn recv_frame_into<'b>(&mut self, buf: &'b mut [u8]) -> io::Result<Frame<'b>> {
        let n = self.inner.read(buf)?;
        let byte_slice = &buf[..n];
        let frame_slice = Frame::new(byte_slice);

        Ok(frame_slice)
    }

    fn recv_frame(&mut self) -> io::Result<FrameBuf> {
        // FLAGS + (one or eight octets)
        let mut head = [0; 9];

        let head_n = self.inner.read(&mut head)?;

        let phantom_frame = Frame::new(&head as &[_]);
        let size = phantom_frame.size().unwrap();

        let frame_buf = if size > head.len() {
            let mut tail = Vec::with_capacity(size - head.len());
            let tail_n = self.inner.read(tail.as_mut_slice())?;

            let mut data = head[..head_n].to_vec();
            data.extend_from_slice(&tail.as_slice()[..tail_n]);

            FrameBuf::new(data)
        } else {
            FrameBuf::new(head[..head_n].to_vec())
        };

        Ok(frame_buf)
    }

    fn bind(_: &str) -> io::Result<Self> {
        unimplemented!()
    }

    fn connect(address: &str) -> io::Result<Self> {
        let inner = crate::stream::Stream::connected(address);
        Ok(Self { inner })
    }
}
