use std::io;

use zedmq::prelude::*;

fn main() -> io::Result<()> {
    let address = String::from("127.0.0.1:8989");
    let req = {
        let socket = zmq::Context::new().socket(zmq::REQ).unwrap();
        socket.bind(format!("tcp://{}", address).as_str()).unwrap();
        socket
    };

    let rep = Rep::connect(address.as_str()).unwrap();

    req.send_multipart(vec![vec![0xFF]], 0x00).unwrap();

    let (x, rep) = dbg!(rep.recv()).unwrap();
    let _ = dbg!(rep.send(x));

    let _ = dbg!(req.recv_multipart(0x00));

    Ok(())
}
