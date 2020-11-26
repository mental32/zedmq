use std::{collections::HashMap, fmt};

use super::Frame;

// -- PropertyIterator<'a>

struct PropertyIterator<'a, I> where I: Iterator<Item = (usize, &'a u8)> {
    inner: &'a Command<'a>,
    cursor: I
}

impl<'a, I> Iterator for PropertyIterator<'a, I> where I: Iterator<Item = (usize, &'a u8)> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        let (name_idx, name_size) = self.cursor.next()?;

        // Jump over the name chunk so that the field is next.
        for _ in 0..(*name_size) {
            let _ = self.cursor.next().expect("Expected to jump over a name chunk byte...");
        }

        // Extract out the name str.
        let name_start = name_idx + 1;
        let name = {
            let range = name_start..(name_start + *name_size as usize);
            let slice = &self.inner.frame.bytes[range];

            std::str::from_utf8(slice).unwrap_or("INVALID.UTF-8")
        };

        // Read four octets into an array and cast it to a u32
        // It's written like this so that the iterator is advanced simultaneously.
        let (field_idx, field_size) = {
            let mut field_size = [0u8; 4];
            let mut field_idx = 0;

            for idx in 0..4 {
                let (pos, byte) = self.cursor
                    .next()
                    .map(|(idx, n)| (idx, *n))
                    .expect("Unexpected EOF");
                field_idx = pos;
                field_size[idx] = byte;
            }

            (field_idx + 1, u32::from_be_bytes(field_size) as usize)
        };

        // And now slice out the field.
        let field = {
            let range = field_idx..(field_idx + field_size as usize);
            let slice = &self.inner.frame.bytes[range];

            std::str::from_utf8(slice).unwrap_or("INVALID.UTF-8")
        };

        // Finally jump over the current field chunk.
        for _ in 0..field_size {
            let _ = self.cursor.next();
        }

        Some((name, field))
    }
}

// -- Command<'_>

pub struct Command<'a> {
    pub(crate) frame: Frame<'a>,
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

    /// Get an iterator over the NULL properties of this command.
    ///
    /// This frame is only sent once after a handshake only if the security
    /// mechanism is NULL.
    #[inline]
    pub fn null_ready_properties(&self) -> Option<impl Iterator<Item = (&str, &str)>> {
        if self.name() != "READY" {
            return None;
        } 

        let cursor = self
            .frame
            .bytes
            .iter()
            .enumerate()
            // Skip ahead to the command-metadata/properties index
            // which is calculated based of frame size.
            .skip(if self.frame.bytes[0] == 0x4 { 3 } else { 10 } + self.name().len());

        let it = PropertyIterator { inner: self, cursor };

        Some(it)
    }
}

impl<'a> fmt::Debug for Command<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "Command {{ name: {:#?}, properties: {:#?} }}",
            self.name(),
            self.null_ready_properties().map(|it| it.collect::<Vec<_>>())
        ))
    }
}
