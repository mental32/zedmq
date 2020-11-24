use std::{collections::HashMap, fmt};

use super::Frame;

// -- Command

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

    pub fn properties(&self) -> HashMap<&str, &str> {
        let idx = if self.frame.bytes[0] == 0x4 { 3 } else { 10 } + self.name().len();

        let mut cursor = self.frame.bytes.iter().enumerate().skip(idx);
        let mut properties = HashMap::new();

        eprintln!("{:?}", &self.frame.bytes);

        while let Some((name_idx, name_size)) = cursor.next() {
            let name_as_bytes =
                &self.frame.bytes[(name_idx + 1)..(name_idx + 1 + *name_size as usize)];
            let name = std::str::from_utf8(name_as_bytes).unwrap_or("INVALID UTF-8");

            dbg!(name);

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
