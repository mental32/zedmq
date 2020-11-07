use std::io;

use zedmq::prelude::*;

fn main() -> io::Result<()> {
    let address = std::env::var("ADDRESS").unwrap();
    let mut socket = dbg!(<Pull as Socket>::connect(address.as_str()))?;

    while dbg!(socket.recv()).is_ok() {}

    Ok(())
}
