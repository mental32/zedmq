mod builder;

mod router;

use builder::SocketBuilder;
use router::RouterType;

pub struct Socket;

impl Socket {
    pub fn router() -> SocketBuilder<RouterType> {
        SocketBuilder::new()
    }
}
