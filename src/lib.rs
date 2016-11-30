#[macro_use]
extern crate nom;
extern crate twoway;

use nom::{alpha, digit};
use std::borrow::Cow;

#[derive(Debug, PartialEq, Eq)]
pub enum Prefix<'a> {
    Server(&'a [u8]),
    User {
        nick: &'a [u8],
        user: Option<&'a [u8]>,
        host: Option<&'a [u8]>,
    },
    Implicit,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Command<'a> {
    Numeric(&'a [u8]),
    String(&'a [u8]),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Message<'a> {
    pub tags: Vec<(&'a [u8], Option<Cow<'a, [u8]>>)>,
    pub prefix: Prefix<'a>,
    pub command: Command<'a>,
    pub params: Vec<&'a [u8]>,
}

fn unescape_value(value: &[u8]) -> Cow<[u8]> {
    const ESCAPE_SEQUENCES: [(&'static [u8], u8); 5] =
        [(b"\\:", b';'), (b"\\s", b' '), (b"\\\\", b'\\'), (b"\\r", b'\r'), (b"\\n", b'\n')];

    let mut value = Cow::Borrowed(value);
    for &(pattern, replacement) in &ESCAPE_SEQUENCES {
        let mut start = 0;
        loop {
            match twoway::find_bytes(&value[start..], pattern) {
                Some(idx) => {
                    let idx = start + idx;
                    let value = value.to_mut();
                    drop(value.drain(idx + 1..idx + pattern.len()));
                    value[idx] = replacement;
                    start = idx + 1;
                }
                None => break,
            }
        }
    }

    value
}

named!(host<&[u8]>,
    alt!(hostname | hostaddr)
);

named!(hostname<&[u8]>,
    recognize!(
        separated_nonempty_list!(
            tag!(b"."),
            take_while!(call!(|b| nom::is_alphanumeric(b) || b == b'-' || b == b'_')))
    )
);

named!(hostaddr<&[u8]>,
    alt!(ip4addr | ip6addr)
);

named!(ip4addr<&[u8]>,
    is_a!(&b"0123456789."[..])
);

named!(ip6addr<&[u8]>,
    is_a!(&b"0123456789abcdefABCDEF:."[..])
);

named!(tags<Vec<(&[u8], Option<Cow<[u8]> >)> >,
    chain!(
        tag!(b"@") ~
        tags: separated_nonempty_list!(tag!(b";"), tag) ~
        tag!(b" "),
        || tags
    )
);

named!(tag<(&[u8], Option<Cow<[u8]>>)>,
    chain!(
        key: recognize!(
            chain!(
                opt!(
                    chain!(
                        host ~
                        tag!(b"/"),
                        || ()
                    )
                ) ~
                take_while!(call!(|b| nom::is_alphanumeric(b) || b == b'-')),
                || ()
            )
        ) ~
        value: opt!(
            chain!(
                tag!(b"=") ~
                value: is_not!(&b"\0\r\n; "[..]),
                || unescape_value(value)
            )
        ),
        || (key, value)
    )
);

/// Checks whether `b` is any of ``[\]`_^{|}``
fn is_special(b: u8) -> bool {
    0x5B <= b && b <= 0x60 || 0x7B <= b && b <= 0x7D
}

named!(nickname<&[u8]>,
    take_while!(call!(|b| nom::is_alphanumeric(b) || is_special(b)))
);

named!(user<&[u8]>,
    is_not!(&b"\0\r\n @"[..])
);

named!(prefix<Prefix>,
    chain!(
        tag!(b":") ~
        prefix: alt!(
            chain!(
                host: hostname ~
                tag!(b" "),
                || Prefix::Server(host)
            ) |
            chain!(
                nick: nickname ~
                user: opt!(
                    chain!(
                        tag!(b"!") ~
                        user: user,
                        || user
                    )
                ) ~
                host: opt!(
                    chain!(
                        tag!(b"@") ~
                        host: host,
                        || host
                    )
                ) ~
                tag!(b" "),
                || Prefix::User {
                    nick: nick,
                    user: user,
                    host: host,
                }
            )
        ),
        || prefix
    )
);

named!(command<Command>,
    alt!(
        map!(digit, Command::Numeric) |
        map!(alpha, Command::String)
    )
);

named!(params<Vec<&[u8]> >,
    chain!(
        tag!(b" ") ~
        params: separated_list!(tag!(b" "), middle) ~
        trailing: opt!(
            chain!(
                tag!(b" :") ~
                trailing: trailing,
                || trailing
            )
        ),
        || {
            if let Some(trailing) = trailing {
                let mut params = params;
                params.push(trailing);
                params
            } else {
                params
            }
        }
    )
);

named!(middle<&[u8]>,
    recognize!(
        chain!(
            is_not!(&b"\0\r\n :"[..]) ~
            opt!(is_not!(&b"\0\r\n "[..])),
            || ()
        )
    )
);

named!(trailing<&[u8]>,
    is_not!(&b"\0\r\n"[..])
);

named!(pub message<Message>,
    chain!(
        tags: opt!(tags) ~
        prefix: opt!(prefix) ~
        command: command ~
        params: opt!(params) ~
        tag!(b"\r\n"),
        || {
            Message {
                tags: tags.unwrap_or_else(Vec::new),
                prefix: prefix.unwrap_or(Prefix::Implicit),
                command: command,
                params: params.unwrap_or_else(Vec::new),
            }
        }
    )
);

/// Example commands and responses from https://dev.twitch.tv/docs/irc/
#[test]
fn twitch_examples() {
    assert_eq!(message(b"PASS oauth:twitch_oauth_token\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Implicit,
        command: Command::String(b"PASS"),
        params: vec![b"oauth:twitch_oauth_token"],
    }));
    assert_eq!(message(b"NICK twitch_username\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Implicit,
        command: Command::String(b"NICK"),
        params: vec![b"twitch_username"],
    }));
    assert_eq!(message(b":tmi.twitch.tv 001 twitch_username :Welcome, GLHF!\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"001"),
        params: vec![b"twitch_username", b"Welcome, GLHF!"],
    }));
    assert_eq!(message(b":tmi.twitch.tv 002 twitch_username :Your host is tmi.twitch.tv\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"002"),
        params: vec![b"twitch_username", b"Your host is tmi.twitch.tv"],
    }));
    assert_eq!(message(b":tmi.twitch.tv 003 twitch_username :This server is rather new\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"003"),
        params: vec![b"twitch_username", b"This server is rather new"],
    }));
    assert_eq!(message(b":tmi.twitch.tv 004 twitch_username :-\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"004"),
        params: vec![b"twitch_username", b"-"],
    }));
    assert_eq!(message(b":tmi.twitch.tv 375 twitch_username :-\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"375"),
        params: vec![b"twitch_username", b"-"],
    }));
    assert_eq!(message(b":tmi.twitch.tv 372 twitch_username :You are in a maze of twisty passages, all alike.\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"372"),
        params: vec![b"twitch_username", b"You are in a maze of twisty passages, all alike."],
    }));
    assert_eq!(message(b":tmi.twitch.tv 376 twitch_username :>\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"376"),
        params: vec![b"twitch_username", b">"],
    }));
    assert_eq!(message(b"WHO #channel\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Implicit,
        command: Command::String(b"WHO"),
        params: vec![b"#channel"],
    }));
    assert_eq!(message(b":tmi.twitch.tv 421 twitch_username WHO :Unknown command\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"421"),
        params: vec![b"twitch_username", b"WHO", b"Unknown command"],
    }));
    assert_eq!(message(b"JOIN #channel\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Implicit,
        command: Command::String(b"JOIN"),
        params: vec![b"#channel"],
    }));
    assert_eq!(message(b":twitch_username!twitch_username@twitch_username.tmi.twitch.tv JOIN #channel\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::User {
            nick: b"twitch_username",
            user: Some(b"twitch_username"),
            host: Some(b"twitch_username.tmi.twitch.tv"),
        },
        command: Command::String(b"JOIN"),
        params: vec![b"#channel"],
    }));
    assert_eq!(message(b":twitch_username.tmi.twitch.tv 353 twitch_username = #channel :twitch_username\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"twitch_username.tmi.twitch.tv"),
        command: Command::Numeric(b"353"),
        params: vec![b"twitch_username", b"=", b"#channel", b"twitch_username"],
    }));
    assert_eq!(message(b":twitch_username.tmi.twitch.tv 366 twitch_username #channel :End of /NAMES list\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"twitch_username.tmi.twitch.tv"),
        command: Command::Numeric(b"366"),
        params: vec![b"twitch_username", b"#channel", b"End of /NAMES list"],
    }));
    assert_eq!(message(b"PART #channel\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Implicit,
        command: Command::String(b"PART"),
        params: vec![b"#channel"],
    }));
    assert_eq!(message(b":twitch_username!twitch_username@twitch_username.tmi.twitch.tv PART #channel\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"003"),
        params: vec![b"twitch_username", b"This server is rather new"],
    }));
    assert_eq!(message(b":twitch_username!twitch_username@twitch_username.tmi.twitch.tv PRIVMSG #channel :message here\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"003"),
        params: vec![b"twitch_username", b"This server is rather new"],
    }));
    assert_eq!(message(b"CAP REQ :twitch.tv/membership\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"003"),
        params: vec![b"twitch_username", b"This server is rather new"],
    }));
    assert_eq!(message(b":tmi.twitch.tv CAP * ACK :twitch.tv/membership\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"003"),
        params: vec![b"twitch_username", b"This server is rather new"],
    }));
    assert_eq!(message(b":twitch_username.tmi.twitch.tv 353 twitch_username = #channel :twitch_username user2 user3\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"003"),
        params: vec![b"twitch_username", b"This server is rather new"],
    }));
    assert_eq!(message(b":twitch_username.tmi.twitch.tv 353 twitch_username = #channel :user5 user6 nicknameN\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"003"),
        params: vec![b"twitch_username", b"This server is rather new"],
    }));
    assert_eq!(message(b":twitch_username.tmi.twitch.tv 366 twitch_username #channel :End of /NAMES list\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"003"),
        params: vec![b"twitch_username", b"This server is rather new"],
    }));
    assert_eq!(message(b":twitch_username!twitch_username@twitch_username.tmi.twitch.tv JOIN #channel\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"003"),
        params: vec![b"twitch_username", b"This server is rather new"],
    }));
    assert_eq!(message(b":twitch_username!twitch_username@twitch_username.tmi.twitch.tv PART #channel\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"003"),
        params: vec![b"twitch_username", b"This server is rather new"],
    }));
    assert_eq!(message(b":jtv MODE #channel +o operator_user\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"003"),
        params: vec![b"twitch_username", b"This server is rather new"],
    }));
    assert_eq!(message(b":jtv MODE #channel -o operator_user\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"003"),
        params: vec![b"twitch_username", b"This server is rather new"],
    }));
    assert_eq!(message(b"CAP REQ :twitch.tv/commands\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"003"),
        params: vec![b"twitch_username", b"This server is rather new"],
    }));
    assert_eq!(message(b":tmi.twitch.tv CAP * ACK :twitch.tv/commands\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"003"),
        params: vec![b"twitch_username", b"This server is rather new"],
    }));
    assert_eq!(message(b"@msg-id=slow_off :tmi.twitch.tv NOTICE #channel :This room is no longer in slow mode.\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"003"),
        params: vec![b"twitch_username", b"This server is rather new"],
    }));
    assert_eq!(message(b":tmi.twitch.tv HOSTTARGET #hosting_channel :target_channel 99999\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"003"),
        params: vec![b"twitch_username", b"This server is rather new"],
    }));
    assert_eq!(message(b":tmi.twitch.tv HOSTTARGET #hosting_channel :- 99999\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"003"),
        params: vec![b"twitch_username", b"This server is rather new"],
    }));
    assert_eq!(message(b":tmi.twitch.tv CLEARCHAT #channel :twitch_username\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"003"),
        params: vec![b"twitch_username", b"This server is rather new"],
    }));
    assert_eq!(message(b":tmi.twitch.tv CLEARCHAT #channel\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"003"),
        params: vec![b"twitch_username", b"This server is rather new"],
    }));
    assert_eq!(message(b":tmi.twitch.tv USERSTATE #channel\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"003"),
        params: vec![b"twitch_username", b"This server is rather new"],
    }));
    assert_eq!(message(b":tmi.twitch.tv ROOMSTATE #channel\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"003"),
        params: vec![b"twitch_username", b"This server is rather new"],
    }));
    assert_eq!(message(b":tmi.twitch.tv USERNOTICE #channel :message\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"003"),
        params: vec![b"twitch_username", b"This server is rather new"],
    }));
    assert_eq!(message(b"CAP REQ :twitch.tv/tags\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"003"),
        params: vec![b"twitch_username", b"This server is rather new"],
    }));
    assert_eq!(message(b":tmi.twitch.tv CAP * ACK :twitch.tv/tags\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"003"),
        params: vec![b"twitch_username", b"This server is rather new"],
    }));
    assert_eq!(message(b"@badges=global_mod/1,turbo/1;color=#0D4200;display-name=TWITCH_UserNaME;emotes=25:0-4,12-16/1902:6-10;mod=0;room-id=1337;subscriber=0;turbo=1;user-id=1337;user-type=global_mod :twitch_username!twitch_username@twitch_username.tmi.twitch.tv PRIVMSG #channel :Kappa Keepo Kappa\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"003"),
        params: vec![b"twitch_username", b"This server is rather new"],
    }));
    assert_eq!(message(b"@badges=staff/1,bits/1000;bits=100;color=;display-name=TWITCH_UserNaME;emotes=;id=b34ccfc7-4977-403a-8a94-33c6bac34fb8;mod=0;room-id=1337;subscriber=0;turbo=1;user-id=1337;user-type=staff :twitch_username!twitch_username@twitch_username.tmi.twitch.tv PRIVMSG #channel :cheer100\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"003"),
        params: vec![b"twitch_username", b"This server is rather new"],
    }));
    assert_eq!(message(b"@color=#0D4200;display-name=TWITCH_UserNaME;emote-sets=0,33,50,237,793,2126,3517,4578,5569,9400,10337,12239;mod=1;subscriber=1;turbo=1;user-type=staff :tmi.twitch.tv USERSTATE #channel\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"003"),
        params: vec![b"twitch_username", b"This server is rather new"],
    }));
    assert_eq!(message(b"@color=#0D4200;display-name=TWITCH_UserNaME;emote-sets=0,33,50,237,793,2126,3517,4578,5569,9400,10337,12239;turbo=0;user-id=1337;user-type=admin :tmi.twitch.tv GLOBALUSERSTATE\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"003"),
        params: vec![b"twitch_username", b"This server is rather new"],
    }));
    assert_eq!(message(b"@broadcaster-lang=;r9k=0;slow=0;subs-only=0 :tmi.twitch.tv ROOMSTATE #channel\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"003"),
        params: vec![b"twitch_username", b"This server is rather new"],
    }));
    assert_eq!(message(b"@slow=10 :tmi.twitch.tv ROOMSTATE #channel\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"003"),
        params: vec![b"twitch_username", b"This server is rather new"],
    }));
    assert_eq!(message(b"@badges=staff/1,broadcaster/1,turbo/1;color=#008000;display-name=TWITCH_UserName;emotes=;mod=0;msg-id=resub;msg-param-months=6;room-id=1337;subscriber=1;system-msg=TWITCH_UserName\\shas\\ssubscribed\\sfor\\s6\\smonths!;login=twitch_username;turbo=1;user-id=1337;user-type=staff :tmi.twitch.tv USERNOTICE #channel :Great stream -- keep it up!\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"003"),
        params: vec![b"twitch_username", b"This server is rather new"],
    }));
    assert_eq!(message(b"@badges=staff/1,broadcaster/1,turbo/1;color=#008000;display-name=TWITCH_UserName;emotes=;mod=0;msg-id=resub;msg-param-months=6;room-id=1337;subscriber=1;system-msg=TWITCH_UserName\\shas\\ssubscribed\\sfor\\s6\\smonths!;login=twitch_username;turbo=1;user-id=1337;user-type=staff :tmi.twitch.tv USERNOTICE #channel\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"003"),
        params: vec![b"twitch_username", b"This server is rather new"],
    }));
    assert_eq!(message(b"@ban-duration=1;ban-reason=Follow\\sthe\\srules :tmi.twitch.tv CLEARCHAT #channel :target_username\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"003"),
        params: vec![b"twitch_username", b"This server is rather new"],
    }));
    assert_eq!(message(b"@ban-reason=Follow\\sthe\\srules :tmi.twitch.tv CLEARCHAT #channel :target_username\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Numeric(b"003"),
        params: vec![b"twitch_username", b"This server is rather new"],
    }));
}

/// Examples from http://ircv3.net/specs/core/message-tags-3.2.html
#[test]
fn ircv32_message_tags_examples() {
    assert_eq!(message(b":nick!ident@host.com PRIVMSG me :Hello\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::User {
            nick: b"nick",
            user: Some(b"ident"),
            host: Some(b"host.com"),
        },
        command: Command::String(b"PRIVMSG"),
        params: vec![b"me", b"Hello"],
    }));
    assert_eq!(message(b"@aaa=bbb;ccc;example.com/ddd=eee :nick!ident@host.com PRIVMSG me :Hello\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![
            (b"aaa", Some(Cow::Borrowed(b"bbb"))),
            (b"ccc", None),
            (b"example.com/ddd", Some(Cow::Borrowed(b"eee"))),
        ],
        prefix: Prefix::User {
            nick: b"nick",
            user: Some(b"ident"),
            host: Some(b"host.com"),
        },
        command: Command::String(b"PRIVMSG"),
        params: vec![b"me", b"Hello"],
    }));
}

/// Things that Twitch does differently.
#[test]
fn twitch_pls() {
    // Nickname starting with a digit.
    assert_eq!(message(b":3and4fifths!3and4fifths@3and4fifths.tmi.twitch.tv PRIVMSG #loadingreadyrun :You missed a window to climb through\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::User {
            nick: b"3and4fifths",
            user: Some(b"3and4fifths"),
            host: Some(b"3and4fifths.tmi.twitch.tv"),
        },
        command: Command::String(b"PRIVMSG"),
        params: vec![b"#loadingreadyrun", b"You missed a window to climb through"],
    }));

    // Hostname component ending with an underscore.
    assert_eq!(message(b":featherweight_!featherweight_@featherweight_.tmi.twitch.tv PRIVMSG #loadingreadyrun :Hello human people\r\n"), nom::IResult::Done(&b""[..], Message {
        tags: vec![],
        prefix: Prefix::User {
            nick: b"featherweight_",
            user: Some(b"featherweight_"),
            host: Some(b"featherweight_.tmi.twitch.tv"),
        },
        command: Command::String(b"PRIVMSG"),
        params: vec![b"#loadingreadyrun", b"Hello human people"],
    }));
}

#[test]
fn test_unescape_value() {
    assert_eq!(unescape_value(b"TWITCH_UserName\\shas\\ssubscribed\\sfor\\s6\\smonths!"), &b"TWITCH_UserName has subscribed for 6 months!"[..]);

    assert_eq!(unescape_value(b"\\:\\s\\\\\\r\\n"), &b"; \\\r\n"[..]);
    assert_eq!(unescape_value(b"\\s\\s\\s\\s\\s"), &b"     "[..]);

    match unescape_value(b"no-escape-sequences") {
        Cow::Borrowed(b"no-escape-sequences") => (),
        e => panic!("Value with no escape sequences has changed: {:?}", e),
    }
}
