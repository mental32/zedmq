# zedmq

## A Tiny, Safe, and pure Rust ØMQ/ZMTP library implementation

## Index

* [Brief](#Brief)
* [Examples](#examples)

## Brief

_Zedmq_ is a native implementation of ØMQ in Rust focusing on speed, safety, and
a minimalistic and human-friendly API.

## Examples

## An echoing ROUTER-based server

```rust
use zedmq::prelude::*;

async fn echo_connection(peer: ClientPeer) {
    while let Some(message) = peer.next().await {
        peer.send(message).await;
    }
}

#[tokio::main]
async fn main() {
    let mut router_socket = Socket::router()
        .bind("tcp://127.0.0.1:5678")
        .build()
        .await.unwrap();

    while Some(connection) = router_socket.incoming().next().await {
        tokio::spawn(echo_connection(connection))
    }
}
```
