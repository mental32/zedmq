use std::io;

use zedmq::prelude::*;

fn main() -> io::Result<()> {
    let address = String::from("127.0.0.1:8989");
    let sub = {
        let socket = zmq::Context::new().socket(zmq::SUB).unwrap();
        socket.bind(format!("tcp://{}", address).as_str()).unwrap();
        socket
    };

    sub.set_subscribe(&[]).unwrap();

    eprintln!("Bound SUB socket on {:?}", address);

    let mut pubs: Pub = zedmq::connect("tcp", address.as_str()).unwrap();

    eprintln!("Connected PUB socket to {:?}", address);

    for i in (0..100).step_by(33) {
        pubs.send(vec![vec![i, i << 1, i & 1]]).unwrap();
    }

    pubs.send(vec![vec![69, 4, 20]]).unwrap();

    for _ in 0..5 {
        let _ = dbg!(sub.recv_multipart(0x00)).unwrap();
    }

    std::process::exit(0);
}
