mod builder;

mod router;

use builder::SocketBuilder;
use router::RouterType;

pub struct Socket;

impl Socket {
    pub fn router<'a>() -> SocketBuilder<'a, RouterType> {
        SocketBuilder::new()
    }

    pub fn push<'a>() -> SocketBuilder<'a, RouterType> {
        SocketBuilder::new()
    }

    pub fn pull<'a>() -> SocketBuilder<'a, RouterType> {
        SocketBuilder::new()
    }
}
