# zedmq
## A tiny, safe, pure Rust, ZMQ(ZMTP) library implementation

```rust
use zedmq::prelude::*;

async fn echo_connection(connection: RouterConnection) {
    while let Some(message) = connection.next().await {
        connection.send(message).await;
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
