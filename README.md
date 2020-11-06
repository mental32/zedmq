# zedmq

## A Lightweight, Safe, pure-Rust ØMQ/ZMTP library implementation

## Index

* [Brief](#brief)

### Brief

_Zedmq_ is a native implementation of ØMQ in Rust focusing on:

* being as lightweight as possible.
* being completely safe.
* providing a simple, blocking, obvious API.

## Examples

```rust
use zedmq::prelude::*;

fn main() -> std::io::Result<()> {
    let mut socket = <Pull as Socket>::bind("tcp://127.0.0.1:5678")?;

    while Some(message) = socket.recv() {
        dbg!(message);
    }

    Ok(())
}
```
