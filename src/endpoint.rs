use lazy_static::lazy_static;
use regex::Regex;

use crate::socket::builder::Protocol;

lazy_static! {
    pub(crate) static ref ENDPOINT_REGEX: Regex = Regex::new(r"(?P<protocol>(tcp|ipc|))://(?P<address>(\d{1,4}\.\d{1,4}\.\d{1,4}\.\d{1,4}))(:?(?P<port>\d+))").unwrap();
}

pub(crate) fn parse(st: &str) -> Option<(Protocol, String, u16)> {
    let captures = ENDPOINT_REGEX.captures(st)?;

    let protocol = if let Some(m) = captures.name("protocol") {
        match m.as_str() {
            "tcp" => Protocol::Tcp,
            _ => panic!("oof"),
        }
    } else {
        panic!("oof");
    };

    let address = captures.name("address")?.as_str().to_string();
    let port = captures
        .name("port")
        .map(|p| p.as_str().parse::<u16>().ok())
        .flatten()
        .unwrap_or(0);

    Some((protocol, address, port))
}
