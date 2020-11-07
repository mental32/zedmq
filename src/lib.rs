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
//!
//! ### Brief
//!
//! _Zedmq_ is a native implementation of ØMQ in Rust focusing on:
//!
//! * being as lightweight as possible.
//! * being completely safe.
//! * providing a simple, blocking, obvious API.
//!
//! ## Examples
//!
//! ```rust
//! use zedmq::prelude::*;
//!
//! fn main() -> std::io::Result<()> {
//!     let mut socket = <Pull as Socket>::bind("127.0.0.1:5678")?;
//!
//!     while Ok(message) = socket.recv() {
//!         dbg!(message);
//!     }
//!
//!     Ok(())
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod codec;
mod socket_type;
mod stream;

/// The prelude.
pub mod prelude {
    pub use crate::socket_type::{pull_t::Pull, Socket};
}
