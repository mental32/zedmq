
use std::{cell::Cell, collections::hash_map::DefaultHasher};
use std::hash::Hasher;
use std::io::{self, Write};
use std::{convert::TryInto, hash::Hash};

use super::{LazyMessage, Socket};
use crate::{codec::FrameBuf, stream::Stream};

#[derive(Clone, Debug)]
enum SubscriptionTopic {
    /// An empty topic (matches everything.)
    Empty,

    /// A literal topic is any topic 8 bytes or smaller.
    ///
    /// We store the literal to avoid extrenious hashing of small prefixes.
    ///
    Literal([u8; 8]),

    /// A hashed topic is the hash of any topic larger than 8 bytes.
    ///
    /// It matches if the hash of the first `length` bytes slice matches
    /// `value`.
    ///
    Hashed { value: u64, length: u8 },
}

/// A ZMQ SUB socket.
pub struct Sub {
    inner: Cell<Stream>,
    topics: Vec<SubscriptionTopic>,
}

impl Sub {
    /// Block until a handshake has succeeded with `address`.
    pub fn connect(address: &str) -> io::Result<Self> {
        <Self as Socket>::connect(address)
    }

    /// Subscribe to a topic.
    pub fn subscribe(&mut self, topic: &[u8]) -> io::Result<()> {
        // Note down the subscribing topic locally for prefix matching
        // when receiving (its a new block because I wanted to reuse "topic" as a name.)
        {
            let slim_topic: Result<[u8; 8], _> = topic.try_into();
            let topic_entry = match (topic.len(), slim_topic) {
                (0, _) => SubscriptionTopic::Empty,
                (_, Ok(slim)) => SubscriptionTopic::Literal(slim),
                (length, _) => {
                    let mut s = DefaultHasher::new();
                    topic.hash(&mut s);
                    let value = s.finish();
                    let length = length
                        .try_into()
                        .expect("Subscription topics can only take 255 bytes maximum");
                    SubscriptionTopic::Hashed { value, length }
                }
            };

            self.topics.push(topic_entry);
        }

        let subscribe = if false {
            // The below code is acceptable for ZMTP 3.1 but not for 3.0 (which is what we are by default.)

            let mut subscribe = vec![
                0x4, // SHORT COMMAND
                0x0, // LENGTH OF FRAME
                // subscribe tag `0xd0 | "SUBSCRIBE".len()`
                // don't ask me why there's a 0xd0 in there
                0xd9,
            ];

            subscribe.extend_from_slice("SUBSCRIBE".as_bytes());
            subscribe.extend_from_slice(topic);
            subscribe[1] = subscribe.len() as u8;

            subscribe
        } else {
            let mut subscribe = vec![0x00, 0xFF, 0x1];

            subscribe.extend_from_slice(&topic);
            subscribe[1] = 1 + topic.len() as u8;
            subscribe
        };

        self.inner.get_mut().ensure_connected().write(&subscribe).map(|_| ())
    }

    /// Recieve a message that matches a subscribed topic prefix.
    #[inline]
    pub fn recv(&mut self) -> io::Result<Vec<Vec<u8>>> {
        fn topic_prefix_match(expected: &SubscriptionTopic, bytes: &[u8]) -> bool {
            match expected {
                SubscriptionTopic::Empty => true,
                SubscriptionTopic::Literal(sl) => bytes.starts_with(sl),
                SubscriptionTopic::Hashed { value, length } => {
                    let mut s = DefaultHasher::new();
                    let tail = &bytes[..(*length as usize)];
                    tail.hash(&mut s);
                    s.finish() == *value
                }
            }
        }

        let stream = self.inner.get_mut();

        loop {
            let mut stream = LazyMessage { stream, witness: false }.fuse();
            let first_frame = stream.next().expect("There should always be one frame in a message.").unwrap();
            let frame = first_frame.as_frame().try_into_message().unwrap();

            let prefix_match = |topic| { topic_prefix_match(topic, &frame.body()) };

            if self.topics.iter().any(prefix_match) {
                let collected = if !frame.is_last() {
                    stream.map(|frame| frame.unwrap().into()).collect()
                } else {
                    vec![first_frame.into()]
                };
 
                return Ok(collected);
            }
        }
    }

    /// Receive a multipart message without performing prefix checks.
    #[inline]
    pub fn recv_unchecked(&mut self) -> io::Result<Vec<Vec<u8>>> {
        <Self as Socket>::recv(self)
    }
}

impl Socket for Sub {
    fn bind(_: &str) -> io::Result<Self> {
        unimplemented!()
    }

    fn connect(address: &str) -> io::Result<Self> {
        let inner = Stream::connected("SUB", address);
        Ok(Self {
            inner: Cell::new(inner),
            topics: vec![],
        })
    }

    fn stream(&mut self) -> &mut crate::stream::Stream {
        self.inner.get_mut()
    }
}
