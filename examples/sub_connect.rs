use std::{io, thread, time::Duration};

use zedmq::prelude::*;

fn main() -> io::Result<()> {
    let address = String::from("127.0.0.1:8989");
    let pubs = {
        let socket = zmq::Context::new().socket(zmq::PUB).unwrap();
        socket.bind(format!("tcp://{}", address).as_str()).unwrap();
        socket
    };

    eprintln!("Bound PUB socket on {:?}", address);

    let mut sub: Sub = zedmq::connect("tcp", address.as_str()).unwrap();

    eprintln!("Connected SUB socket to {:?}", address);

    sub.subscribe(&[]).unwrap();

    eprintln!("Subscribed with empty prefix");


    thread::spawn(move || loop {
        pubs.send_multipart(vec![vec![0xFF]], 0x00).unwrap();
        eprintln!("Tick.");
        thread::sleep(Duration::from_secs(1));
    });

    let _ = dbg!(sub.recv()).unwrap();

    std::process::exit(0);
}
