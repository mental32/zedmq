use std::io;

use zedmq::prelude::*;

fn main() -> io::Result<()> {
    let address = std::env::var("ADDRESS").unwrap();
    let mut socket = Pull::connect(address.as_str())?;

    while let Ok(message) = socket.recv() {
        dbg!(message);
    }

    Ok(())
}
