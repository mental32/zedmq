use std::convert::TryInto;

use super::{Command, Message};

// -- FrameBuf

/// An owned and growable frame.
#[derive(Debug, PartialEq)]
pub struct FrameBuf {
    pub(crate) bytes: Vec<u8>,
}

impl AsRef<[u8]> for FrameBuf {
    fn as_ref(&self) -> &[u8] {
        self.bytes.as_slice()
    }
}

impl From<Vec<u8>> for FrameBuf {
    fn from(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
}

impl From<FrameBuf> for Vec<u8> {
    fn from(FrameBuf { bytes }: FrameBuf) -> Self {
        bytes
    }
}

impl FrameBuf {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    pub fn short_command<'a, I>(name: &str, properties: Option<I>) -> Self
    where
        I: IntoIterator<Item = (&'a str, &'a str)>,
    {
        assert_eq!(name.len(), name.as_bytes().len());

        let mut bytes = vec![
            // SHORT COMMAND
            0x4,
            // LENGTH OF ENTIRE FRAME
            0x00,
            // name as cstr length
            name.len() as u8,
        ];

        let body: Vec<u8> = match properties {
            None => vec![],
            Some(it) => {
                let mut payload = vec![];

                // For every property:
                //
                //  1) push the name length as u8
                //  2) extend with the name bytes
                //  3) push the field length as u64 in network byte order
                //  4) extend with the field bytes
                for (st, field) in it.into_iter() {
                    match st.len().try_into() {
                        Ok(length) => payload.push(length),
                        Err(_) => panic!("property names can not be longer than 255 bytes."),
                    }

                    payload.extend_from_slice(st.as_bytes());
                    payload.extend_from_slice(&u32::to_be_bytes(field.len() as u32) as &[_]);
                    payload.extend_from_slice(field.as_bytes());
                }

                payload
            }
        };

        bytes.extend_from_slice(name.as_bytes());
        bytes.extend_from_slice(body.as_slice());

        // Size is size_of(frame) - 2
        // (don't include the flags byte or the size byte itself.)
        bytes[1] = (bytes.len() - 2).try_into().unwrap();

        Self { bytes }
    }

    pub fn as_frame<'a>(&'a self) -> Frame<'a> {
        Frame::new(self.bytes.as_slice())
    }
}

// -- Frame<'a>

/// A slice of frame (akin to `str` or `Path`)
#[derive(Debug, PartialEq)]
pub struct Frame<'a> {
    pub(crate) bytes: &'a [u8],
}

impl<'a> Frame<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    pub fn try_into_command(self) -> Option<Command<'a>> {
        match self.kind()? {
            FrameKind::Command => Some(Command { frame: self }),
            _ => None,
        }
    }

    pub fn try_into_message(self) -> Option<Message<'a>> {
        match self.kind()? {
            FrameKind::MessagePart => Some(Message {
                frame: self,
                is_last: false,
            }),
            FrameKind::MessageTail => Some(Message {
                frame: self,
                is_last: true,
            }),
            _ => None,
        }
    }

    /// Get the size of the frame.
    pub fn size(&self) -> Option<usize> {
        match self.bytes.get(0)? {
            0x0 | 0x1 | 0x4 => Some(*self.bytes.get(1)? as usize),

            0x2 | 0x3 | 0x6 => {
                let slice = self.bytes.get(1..)?.try_into().ok()?;
                let size = u64::from_be_bytes(slice);
                Some(size as usize)
            }

            _ => None,
        }
    }

    /// The type of frame.
    pub fn kind(&self) -> Option<FrameKind> {
        let kind = match self.bytes.get(0)? {
            // short/long message tail.
            0x0 | 0x2 => FrameKind::MessageTail,

            // short/long message part.
            0x1 | 0x3 => FrameKind::MessagePart,

            // short/long command.part.
            // [frame_tag, ...(frame_size{1u8} | frame_size{8u8}), size, ...command_string{size}]
            0x4 | 0x6 => FrameKind::Command,

            // All the rest...
            _ => return None,
        };

        Some(kind)
    }
}

/// The various types a frame can be.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum FrameKind {
    /// A command frame.
    Command,

    /// A single part (frame) of a message (multipart)
    MessagePart,

    /// The last frame of a multipart message or a singular frame of a non-multipart.
    MessageTail,
}

impl<'a> From<&'a [u8]> for Frame<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }
}
