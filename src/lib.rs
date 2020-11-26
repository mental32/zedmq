//! > Note that this library is in the very early stages of development!
//! > Anything and everything may change!
//!
//! # zedmq
//!
//! ## A Lightweight, Safe, pure-Rust ØMQ/ZMTP library implementation
//!
//! ## Index
//!
//! * [Brief](#brief)
//! * [Examples](#examples)
//!
//! ### Brief
//!
//! _Zedmq_ is a native implementation of ØMQ in Rust focusing on:
//!
//! * being as lightweight as possible.
//! * being completely safe.
//! * providing a simple, blocking, obvious API.
//!
//! This library is lazy and blocking, no work is done unless a call to a
//! `recv` or `send` is made. There is no "background" thread or task accepting
//! new connections or performing reconnections on the users behalf
//! consequently there is no shared state or synchronization being performed.
//!
//! #### `Frame<'_>` and `FrameBuf`
//!
//! This library also exposes the underlying ZMQ concept of a frame.
//! Additionally a distinction is made with the `Frame` and `FrameBuf` types
//! for optimization purposes.
//!
//! Conceptually a `Frame<'_>` is equivelent to `&'_ [u8]` or `&'_ str` and
//! the `FrameBuf` equivelent is `Vec<u8>` or `String`. This distinction is
//! made in an attempt to make "zero copy" or "zero heap" practice easier.
//!
//! ### Examples
//!
//! ```rust,no_run
//! use zedmq::prelude::*;
//!
//! fn main() -> std::io::Result<()> {
//!     let mut socket = Pull::connect("127.0.0.1:5678")?;
//!
//!     while let Ok(message) = socket.recv() {
//!         dbg!(message);
//!     }
//!
//!     Ok(())
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// The currently supported ZMQ version.
pub const ZMQ_VERSION: (u8, u8, u8) = (4, 3, 2);

pub(crate) mod codec;
pub(crate) mod socket_type;
pub(crate) mod stream;

/// The prelude.
pub mod prelude {
    pub(crate) use super::*;

    pub use socket_type::{pull_t::Pull, push_t::Push};
}
