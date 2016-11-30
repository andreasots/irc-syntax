#[macro_use]
extern crate nom;

#[derive(Debug, PartialEq, Eq)]
pub enum Prefix<'a> {
    Server(&'a [u8]),
    User {
        nick: &'a [u8],
        user: Option<&'a [u8]>,
        host: Option<&'a [u8]>,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub struct Message<'a> {
    pub tags: Vec<(&'a [u8], &'a [u8])>,
    pub prefix: Option<Prefix<'a>>,
    pub command: &'a [u8],
    pub params: Vec<&'a [u8]>,
}

named!(prefix<Prefix>,
    map!(
        take_until!(&b" "[..]),
        |prefix| Prefix::Server(prefix)
    )
);

named!(command<&[u8]>,
    take_until!(&b" "[..])
);

named!(params<Vec<&[u8]> >,
    chain!(
        tag!(&b" "[..]) ~
        params: take_until!(&b"\r\n"[..]),
        || vec![params]
    )
);

named!(pub message<Message>,
    chain!(
        prefix: opt!(
            chain!(
                tag!(&b":"[..]) ~
                prefix: prefix ~
                tag!(&b" "[..]),
                || prefix
            )
        ) ~
        command: command ~
        params: params ~
        tag!(&b"\r\n"[..]),
        || Message {
            tags: vec![],
            prefix: prefix,
            command: command,
            params: params,
        }
    )
);

#[cfg(test)]
mod tests {
    use super::*;
    use nom::IResult::Done;

    #[test]
    fn connection_start() {
        assert_eq!(message(b"PASS oauth:twitch_oauth_token\r\n"), Done(&b""[..], Message {
            tags: vec![],
            prefix: None,
            command: b"PASS",
            params: vec![b"oauth:twitch_oauth_token"],
        }));
        assert_eq!(message(b"NICK twitch_username\r\n"), Done(&b""[..], Message {
            tags: vec![],
            prefix: None,
            command: b"NICK",
            params: vec![b"twitch_username"],
        }));
        assert_eq!(message(b":tmi.twitch.tv 001 twitch_username :Welcome, GLHF!\r\n"), Done(&b""[..], Message {
            tags: vec![],
            prefix: Some(Prefix::Server(b"tmi.twitch.tv")),
            command: b"001",
            params: vec![b"twitch_username", b"Welcome, GLHF!"],
        }));
    }
}
