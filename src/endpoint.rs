use std::convert::TryFrom;

use logos::Logos;

#[derive(Debug, PartialEq)]
enum ConnectionKind {
    Tcp,
}

impl<'a> TryFrom<&'a str> for ConnectionKind {
    type Error = ();

    fn try_from(st: &'a str) -> Result<Self, Self::Error> {
        match st {
            "tcp" => Ok(Self::Tcp),
            _ => Err(()),
        }
    }
}

#[derive(Logos, Debug, PartialEq)]
enum EndpointParser<'a> {
    #[regex("[A-Za-z]+")]
    Protocol,

    #[token("://")]
    ProtocolDelim,

    #[regex(r"\d+\.\d+\.\d+\.\d+", |lex| if lex.slice() == "*" { None } else { Some(lex.slice()) })]
    Address(Option<&'a str>),

    #[token(":")]
    Colon,

    #[regex(r"\d+", |lex| lex.slice().parse::<usize>().ok())]
    Port(Option<usize>),

    #[error]
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}

pub struct Endpoint<'a> {
    kind: ConnectionKind,
    address: Option<&'a str>,
    port: Option<usize>,
}

impl<'a> Endpoint<'a> {
    pub(crate) fn parse(st: &'a str) -> Result<Self, ()> {
        use EndpointParser::{Address, Colon, Port, Protocol, ProtocolDelim};

        let mut connection_kind = ConnectionKind::Tcp;
        let mut address = None;
        let mut port = None;

        let mut lex = EndpointParser::lexer(st);

        let mut n = 0usize;

        loop {
            let item = if let Some(token) = lex.next() {
                (n, token)
            } else {
                break;
            };

            match item {
                (0, Protocol) => connection_kind = ConnectionKind::try_from(lex.slice()).unwrap(),
                (0, unexpected) => panic!(
                    "Expected a protocol i.e. tcp or udp, instead got={:?}",
                    unexpected
                ),

                (1, ProtocolDelim) => (),
                (1, unexpected) => panic!(
                    "Expected a protocol deliminator ('://'), instead got={:?}",
                    unexpected
                ),

                (2, Address(st)) => address = st,
                (2, unexpected) => panic!(
                    "Expected an address i.e. '127.0.0.1' or '*', instead got={:?}",
                    unexpected
                ),

                (3, Colon) => (),
                (3, unexpected) => panic!("Expected a colom (':'), instead got={:?}", unexpected),

                (4, Port(p)) => port = p,
                (4, unexpected) => panic!(
                    "Expected a port i.e. '8080' or '*', instead got={:?}",
                    unexpected
                ),

                (n, token) => panic!(
                    "Did not expect any more information! instead got={:?} as nth={:?} token",
                    token, n
                ),
            }

            n = n.wrapping_add(1);
        }

        Ok(Self {
            kind: connection_kind,
            address,
            port,
        })
    }
}

impl<'a> TryFrom<&'a str> for Endpoint<'a> {
    type Error = ();

    fn try_from(st: &'a str) -> Result<Self, Self::Error> {
        Self::parse(st)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tcp_loopback_8080() {
        let mut lex = EndpointParser::lexer("tcp://127.0.0.1:8080");

        assert_eq!(lex.next(), Some(EndpointParser::Protocol));
        assert_eq!(lex.slice(), "tcp");

        assert_eq!(lex.next(), Some(EndpointParser::ProtocolDelim));
        assert_eq!(lex.slice(), "://");

        assert_eq!(lex.next(), Some(EndpointParser::Address(Some("127.0.0.1"))));
        assert_eq!(lex.slice(), "127.0.0.1");

        assert_eq!(lex.next(), Some(EndpointParser::Colon));
        assert_eq!(lex.slice(), ":");

        assert_eq!(lex.next(), Some(EndpointParser::Port(Some(8080))));
        assert_eq!(lex.slice(), "8080");

        assert_eq!(lex.next(), None);
    }

    #[test]
    fn endpoint_try_from_tcp_loopback() {
        let endpoint = Endpoint::try_from("tcp://127.0.0.1:8080").unwrap();

        assert_eq!(endpoint.port, Some(8080));
        assert_eq!(endpoint.address, Some("127.0.0.1"));
        assert_eq!(endpoint.kind, ConnectionKind::Tcp);
    }
}
