> Note that this library is in the very early stages of development!
> Anything and everything may change!

# zedmq

## A Lightweight, Safe, pure-Rust ØMQ/ZMTP library implementation

## Index

* [Brief](#brief)
* [Examples](#examples)

### Brief

_Zedmq_ is a native implementation of ØMQ in Rust focusing on:

* being as lightweight as possible.
* being completely safe.
* providing a simple, blocking, obvious API.

This library is lazy and blocking, no work is done unless a call to a
`recv` or `send` is made. There is no "background" thread or task accepting
new connections or performing reconnections on the users behalf
consequently there is no shared state or synchronization being performed.

#### Caveats

Currently this library only supports connecting sockets
over TCP, no binding behaviour is available.

Also only a few socket types have been implemented: REQ, REP, PULL, PUSH,
and SUB (PUB is being worked on).

#### `Frame<'_>` and `FrameBuf`

This library also exposes the underlying ZMQ concept of a frame.
Additionally a distinction is made with the `Frame` and `FrameBuf` types
for optimization purposes.

Conceptually a `Frame<'_>` is equivelent to `&'_ [u8]` or `&'_ str` and
the `FrameBuf` equivelent is `Vec<u8>` or `String`. This distinction is
made in an attempt to make "zero copy" or "zero heap" practice easier.

#### `REQ` and `REP`

The design of `REQ` and `REP` sockets are symetrical and rendered safe
through the use of the type system.

A `Req` socket only has `.connect` and `.send` methods, `.send` of which
consumes the socket and returns a `ReqPending` socket which only has a `.recv`
method which in turn returns a multipart message and `Req` socket tuple.

Same goes for the `Rep` socket except that `Rep` has `.recv` and
`RepPending` has `.send`.

This done on purpose, its value is that there are no accidental footguns
involved with accidentially `.send`ing when you are only allowed to `.recv`
or vice versa. Plus it removes the cost of runtime checking.

### Examples

```rust
use zedmq::prelude::*;

fn main() -> std::io::Result<()> {
    let mut socket: Pull = zedmq::connect("tcp", "127.0.0.1:5678")?;

    while let Ok(message) = socket.recv() {
        dbg!(message);
    }

    Ok(())
}
```

License: MIT
