use std::{collections::HashMap, convert::TryInto, fmt};

// -- FrameBuf

/// An owned and growable frame.
#[derive(Debug, PartialEq)]
pub struct FrameBuf {
    bytes: Vec<u8>,
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
                    payload.push(st.len().try_into().unwrap());
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
    bytes: &'a [u8],
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
                let size = u64::from_be_bytes(self.bytes.get(1..)?.try_into().unwrap()) as usize;
                Some(size)
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

// -- greeting()

/// # Examples
/// ```ignore
/// use crate::codec::greeting;
///
/// let g = greeting();
/// let (left, right) = (&g[..=11], &g[12..]);
/// assert_eq!(left, &[0xFFu8, 0, 0, 0, 0, 0, 0, 0, 0, 0x7F, 3, 0] as &[u8]);
/// assert_eq!(&right[..4], b"NULL" as &[u8]);
/// ```
pub(crate) fn greeting() -> [u8; 64] {
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

// -- Message

pub struct Message<'a> {
    frame: Frame<'a>,
    is_last: bool,
}

impl<'a> Message<'a> {
    #[inline]
    pub fn is_last(&self) -> bool {
        self.is_last
    }

    #[inline]
    pub fn body(&self) -> &'a [u8] {
        let start = if self.frame.bytes[0] == 0x0 || self.frame.bytes[0] == 0x1 {
            2
        } else {
            9
        };
        &self.frame.bytes[start..]
    }
}

// -- Command

pub struct Command<'a> {
    frame: Frame<'a>,
}

impl<'a> Command<'a> {
    pub fn new(frame: Frame<'a>) -> Self {
        Self { frame }
    }

    pub fn name(&self) -> &str {
        let idx = if self.frame.bytes[0] == 0x4 { 2 } else { 9 };
        let size = self.frame.bytes[idx];
        let start = idx + 1;
        let end = start + (size as usize);
        let st = std::str::from_utf8(&self.frame.bytes[start..end]).unwrap();
        st
    }

    pub fn properties(&self) -> HashMap<&str, &str> {
        let idx = if self.frame.bytes[0] == 0x4 { 3 } else { 10 } + self.name().len();

        let mut cursor = self.frame.bytes.iter().enumerate().skip(idx);
        let mut properties = HashMap::new();

        while let Some((name_idx, name_size)) = cursor.next() {
            let name_as_bytes =
                &self.frame.bytes[(name_idx + 1)..(name_idx + 1 + *name_size as usize)];
            let name = std::str::from_utf8(name_as_bytes).unwrap_or("INVALID UTF-8");

            for _ in 0..(*name_size) {
                let _ = cursor.next();
            }

            let (field_idx, field_size) = {
                let mut field_size = [0u8; 4];
                let mut field_idx = 0;

                for idx in 0..4 {
                    let (pos, byte) = cursor
                        .next()
                        .map(|(idx, n)| (idx, *n))
                        .expect("Unexpected EOF");
                    field_idx = pos;
                    field_size[idx] = byte;
                }

                (field_idx + 1, u32::from_be_bytes(field_size) as usize)
            };

            let field_as_bytes = &self.frame.bytes[field_idx..(field_idx + field_size)];
            let field = std::str::from_utf8(field_as_bytes).unwrap_or("INVALID UTF-8");

            properties.insert(name, field);

            for _ in 0..(field_size) {
                let _ = cursor.next();
            }
        }

        assert!(cursor.next().is_none());

        properties
    }
}

impl<'a> fmt::Debug for Command<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "Command {{ name: {:#?}, properties: {:#?} }}",
            self.name(),
            self.properties()
        ))
    }
}
