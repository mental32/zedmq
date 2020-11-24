pub mod command;
pub mod frame;
pub mod message;
pub mod protocol;

use std::convert::TryInto;

pub use self::{command::*, frame::*, message::*, protocol::*};

// -- Greeting

#[derive(Debug, Default)]
pub struct Greeting {
    as_server: bool,
}

impl Greeting {
    pub fn build() -> Self {
        Default::default()
    }

    /// Set the `as_server` field of the greeting.
    pub fn as_server(&mut self, as_server: bool) -> &mut Self {
        self.as_server = as_server;
        self
    }

    pub fn security(&mut self, security: ()) -> &mut Self {
        unimplemented!();
    }

    pub fn into_parts(&self) -> ([u8; 12], [u8; 52]) {
        let raw = self.as_bytes();
        let partial = raw[..=11].try_into().unwrap();
        // dbg!(raw[..=11].len());
        let remaining = raw[12..].try_into().unwrap();
        (partial, remaining)
    }

    /// Serialize the `Greeting` struct into a raw `[u8; 64]` greeting.
    pub fn as_bytes(&self) -> [u8; 64] {
        // TODO: Missing a way to specify the security mechanism (currently NULL) and the as-server field (currently false)
        let mut raw = [0u8; 64];

        // signature
        raw[0] = 0xFF; // signature start
                       // signature padding.
        raw[9] = 0x7F; // signature end

        // version
        raw[10] = 3;
        raw[11] = 0;

        // Security (NULL)
        raw[12] = 0x4E;
        raw[13] = 0x55;
        raw[14] = 0x4C;
        raw[15] = 0x4C;

        // as-server
        raw[32] = self.as_server as u8;

        raw
    }
}

// /// # Examples
// /// ```ignore
// /// use crate::codec::greeting;
// ///
// /// let g = greeting();
// /// let (left, right) = (&g[..=11], &g[12..]);
// /// assert_eq!(left, &[0xFFu8, 0, 0, 0, 0, 0, 0, 0, 0, 0x7F, 3, 0] as &[u8]);
// /// assert_eq!(&right[..4], b"NULL" as &[u8]);
// /// ```
// pub(crate) fn greeting() -> [u8; 64] {
//     // TODO: Missing a way to specify the security mechanism (currently NULL) and the as-server field (currently false)

//     let mut raw = [0u8; 64];

//     raw[0] = 0xFF; // signature start
//                    // signature padding.
//     raw[9] = 0x7F; // signature end
//     raw[10] = 3;
//     raw[11] = 0;

//     // Security
//     raw[12] = 0x4E;
//     raw[13] = 0x55;
//     raw[14] = 0x4C;
//     raw[15] = 0x4C;

//     raw
// }
