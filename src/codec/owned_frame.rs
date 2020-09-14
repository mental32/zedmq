use std::convert::TryFrom;

use super::frame::Frame;

/// Represents an owned ZMTP-described frame.
#[derive(Debug, PartialEq)]
pub struct OwnedFrame(Box<[u8]>);

impl From<Vec<u8>> for OwnedFrame {
    fn from(v: Vec<u8>) -> Self {
        Self(v.into_boxed_slice())
    }
}

impl OwnedFrame {
    pub fn as_ref<'a>(&'a self) -> Frame<'a> {
        Frame::try_from(self.0.as_ref())
            .expect("The underlying frame should've been validated already.")
    }
}
