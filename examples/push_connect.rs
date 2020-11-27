use std::io;

use zedmq::prelude::*;

fn main() -> io::Result<()> {
    let address = std::env::var("ADDRESS").unwrap();
    let mut socket = Push::connect(address.as_str()).unwrap();

    let msg = (b"oof" as &[u8]).to_vec();

    loop {
        let _ = socket.send(vec![msg.clone(), vec![0, 1]]).unwrap();
        println!("Send!");
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
