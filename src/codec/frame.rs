use std::convert::{TryFrom, TryInto};

type Bytes<'a> = &'a [u8];

#[derive(Debug, PartialEq)]
pub enum RawFrame<'a> {
    /// A command frame.
    Command { name: &'a str, body: Bytes<'a> },

    /// A single part (frame) of a message (multipart)
    MessagePart(Bytes<'a>),

    /// The last frame of a multipart message or a singular frame of a non-multipart.
    MessageTail(Bytes<'a>),
}

impl<'a> TryFrom<(u8, &'a [u8])> for RawFrame<'a> {
    type Error = ();

    #[inline]
    fn try_from((kind, bytes): (u8, &'a [u8])) -> Result<Self, Self::Error> {
        let (name, body_start) = match kind {
            // short/long message tail.
            0x0 | 0x2 => return Ok(Self::MessageTail(&bytes[2..])),

            // short/long message part.
            0x1 | 0x3 => return Ok(Self::MessagePart(&bytes[2..])),

            // short/long command.par
            // [frame_tag, ...(frame_size{1u8} | frame_size{8u8}), size, ...command_string{size}]
            0x4 | 0x6 => {
                let idx = if kind == 0x4 { 2 } else { 9 };

                let size = bytes[idx];
                let start = idx + 1;
                let end = start + (size as usize);

                let st = std::str::from_utf8(&bytes[start..end]).map_err(|_| ())?;

                (st, end)
            }

            // All the rest...
            _ => return Err(()), // TODO: Proper error handling.
        };

        // TODO: parse command properties.

        Ok(Self::Command {
            name,
            body: &bytes[body_start..],
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum Frame<'a> {
    Short { size: usize, body: RawFrame<'a> },
    Long { size: usize, body: RawFrame<'a> },
}

impl<'a> TryFrom<&'a [u8]> for Frame<'a> {
    type Error = ();

    #[inline]
    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        let kind = bytes[0];
        let body = RawFrame::try_from((kind, bytes))?;

        let (size, is_short) = match kind {
            0x0 | 0x1 | 0x4 => (bytes[1] as usize, true),

            0x2 | 0x3 | 0x6 => (
                u64::from_be_bytes(bytes[1..].try_into().unwrap()) as usize,
                false,
            ),

            _ => unreachable!("Unrecognizable frame size tag..."),
        };

        let frame = if is_short {
            Self::Short { size, body }
        } else {
            Self::Long { size, body }
        };

        Ok(frame)
    }
}

impl<'a> Frame<'a> {
    /// Copies `self` into a new `Vec`.
    ///
    /// # Examples
    ///
    /// ```
    /// use zedmq::prelude::*;
    ///
    /// let example = Frame::Short { size: 0, body: RawFrame::MessageTail(&[]) };
    /// assert_eq!(example.to_vec().as_slice(), &[0x00, 0x00]);
    /// ```
    #[inline]
    pub fn to_vec(&self) -> Vec<u8> {
        // Using the current `Frame<'_>` api its a pain in the ass to
        // transparently get the current frame as a raw `&[u8]` representation.
        //
        // so we have to manually re-construct them every time...
        let mut raw = vec![];

        match self {
            Self::Short {
                size,
                body: RawFrame::MessageTail(bytes),
            } => {
                raw.extend_from_slice(&[0x0, *size as u8]);
                raw.extend_from_slice(bytes);
            }

            Self::Short {
                size,
                body: RawFrame::MessagePart(bytes),
            } => {
                raw.extend(vec![0x1, *size as u8]);
                raw.extend_from_slice(bytes);
            }

            Self::Long {
                size,
                body: RawFrame::MessageTail(bytes),
            } => {
                raw.extend_from_slice(&[0x2, *size as u8]);
                raw.extend_from_slice(bytes);
            }

            Self::Long {
                size,
                body: RawFrame::MessagePart(bytes),
            } => {
                raw.extend(vec![0x3, *size as u8]);
                raw.extend_from_slice(bytes);
            }

            Self::Short {
                size,
                body: RawFrame::Command { name, body },
            } => {
                raw.extend_from_slice(&[0x4, *size as u8, name.len() as u8]);
                raw.extend_from_slice(name.as_bytes());
                raw.extend_from_slice(body);
            }

            Self::Long {
                size,
                body: RawFrame::Command { name, body },
            } => {
                raw.extend_from_slice(&[0x6, *size as u8, name.len() as u8]);
                raw.extend_from_slice(name.as_bytes());
                raw.extend_from_slice(body);
            }
        };

        raw
    }
}
