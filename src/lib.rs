#![deny(missing_docs)]

//! IRC message parser loosely inspired by [RFC 2812](https://tools.ietf.org/html/rfc2812).

#[macro_use]
extern crate nom;
extern crate twoway;

use nom::{alpha, digit};
use std::borrow::Cow;

/// Trait to abstract over ownership.
pub trait ToMut {
    /// Owned version of `Self`.
    type Owned;
    /// Type from which `Self::Owned` can be mutably borrowed.
    // FIXME?: `type Container: AsMut<Self::Owned>`? But `AsMut<_>` is not implemented for `Cow<_, _>`...
    type Container;

    /// Converts `&mut Self::Container` to a `&mut Self::Owned`.
    fn to_mut(container: &mut Self::Container) -> &mut Self::Owned;
}

impl<'a> ToMut for &'a [u8] {
    type Owned = Vec<u8>;
    type Container = Cow<'a, [u8]>;

    fn to_mut<'b>(container: &'b mut Cow<'a, [u8]>) -> &'b mut Vec<u8> {
        container.to_mut()
    }
}

impl ToMut for Vec<u8> {
    type Owned = Vec<u8>;
    type Container = Vec<u8>;

    fn to_mut(container: &mut Vec<u8>) -> &mut Vec<u8> {
        container
    }
}

impl<'a> ToMut for &'a str {
    type Owned = String;
    type Container = Cow<'a, str>;

    fn to_mut<'b>(container: &'b mut Cow<'a, str>) -> &'b mut String {
        container.to_mut()
    }
}

impl ToMut for String {
    type Owned = String;
    type Container = String;

    fn to_mut(container: &mut String) -> &mut String {
        container
    }
}

/// Message source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Prefix<T> {
    /// Message was sent by a server.
    Server(T),
    /// Message was sent by a user.
    User {
        /// User's nickname.
        nick: T,
        /// User's username.
        user: Option<T>,
        /// User's hostname.
        host: Option<T>,
    },
    /// Prefix was missing.
    Implicit,
}

// FIXME: docs
/// Numeric reply.
#[allow(missing_docs)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Reply {
    WELCOME = 1,
    YOURHOST = 2,
    CREATED = 3,
    MYINFO = 4,
    BOUNCE = 5,
    TRACELINK = 200,
    TRACECONNECTING = 201,
    TRACEHANDSHAKE = 202,
    TRACEUNKNOWN = 203,
    TRACEOPERATOR = 204,
    TRACEUSER = 205,
    TRACESERVER = 206,
    TRACESERVICE = 207,
    TRACENEWTYPE = 208,
    TRACECLASS = 209,
    TRACERECONNECT = 210,
    STATSLINKINFO = 211,
    STATSCOMMANDS = 212,
    STATSCLINE = 213,
    STATSNLINE = 214,
    STATSILINE = 215,
    STATSKLINE = 216,
    STATSQLINE = 217,
    STATSYLINE = 218,
    ENDOFSTATS = 219,
    UMODEIS = 221,
    SERVICEINFO = 231,
    ENDOFSERVICES = 232,
    SERVICE = 233,
    SERVLIST = 234,
    SERVLISTEND = 235,
    STATSVLINE = 240,
    STATSLLINE = 241,
    STATSUPTIME = 242,
    STATSOLINE = 243,
    STATSHLINE = 244,
    STATSPING = 246,
    STATSBLINE = 247,
    STATSDLINE = 250,
    LUSERCLIENT = 251,
    LUSEROP = 252,
    LUSERUNKNOWN = 253,
    LUSERCHANNELS = 254,
    LUSERME = 255,
    ADMINME = 256,
    ADMINLOC1 = 257,
    ADMINLOC2 = 258,
    ADMINEMAIL = 259,
    TRACELOG = 261,
    TRACEEND = 262,
    TRYAGAIN = 263,
    NONE = 300,
    AWAY = 301,
    USERHOST = 302,
    ISON = 303,
    UNAWAY = 305,
    NOWAWAY = 306,
    WHOISUSER = 311,
    WHOISSERVER = 312,
    WHOISOPERATOR = 313,
    WHOWASUSER = 314,
    ENDOFWHO = 315,
    WHOISCHANOP = 316,
    WHOISIDLE = 317,
    ENDOFWHOIS = 318,
    WHOISCHANNELS = 319,
    LISTSTART = 321,
    LIST = 322,
    LISTEND = 323,
    CHANNELMODEIS = 324,
    UNIQOPIS = 325,
    NOTOPIC = 331,
    TOPIC = 332,
    INVITING = 341,
    SUMMONING = 342,
    INVITELIST = 346,
    ENDOFINVITELIST = 347,
    EXCEPTLIST = 348,
    ENDOFEXCEPTLIST = 349,
    VERSION = 351,
    WHOREPLY = 352,
    NAMREPLY = 353,
    KILLDONE = 361,
    CLOSING = 362,
    CLOSEEND = 363,
    LINKS = 364,
    ENDOFLINKS = 365,
    ENDOFNAMES = 366,
    BANLIST = 367,
    ENDOFBANLIST = 368,
    ENDOFWHOWAS = 369,
    INFO = 371,
    MOTD = 372,
    INFOSTART = 373,
    ENDOFINFO = 374,
    MOTDSTART = 375,
    ENDOFMOTD = 376,
    YOUREOPER = 381,
    REHASHING = 382,
    YOURESERVICE = 383,
    MYPORTIS = 384,
    TIME = 391,
    USERSSTART = 392,
    USERS = 393,
    ENDOFUSERS = 394,
    NOUSERS = 395,
}

impl Reply {
    fn from(data: u16) -> Option<Reply> {
        match data {
            1 => Some(Reply::WELCOME),
            2 => Some(Reply::YOURHOST),
            3 => Some(Reply::CREATED),
            4 => Some(Reply::MYINFO),
            5 => Some(Reply::BOUNCE),
            200 => Some(Reply::TRACELINK),
            201 => Some(Reply::TRACECONNECTING),
            202 => Some(Reply::TRACEHANDSHAKE),
            203 => Some(Reply::TRACEUNKNOWN),
            204 => Some(Reply::TRACEOPERATOR),
            205 => Some(Reply::TRACEUSER),
            206 => Some(Reply::TRACESERVER),
            207 => Some(Reply::TRACESERVICE),
            208 => Some(Reply::TRACENEWTYPE),
            209 => Some(Reply::TRACECLASS),
            210 => Some(Reply::TRACERECONNECT),
            211 => Some(Reply::STATSLINKINFO),
            212 => Some(Reply::STATSCOMMANDS),
            213 => Some(Reply::STATSCLINE),
            214 => Some(Reply::STATSNLINE),
            215 => Some(Reply::STATSILINE),
            216 => Some(Reply::STATSKLINE),
            217 => Some(Reply::STATSQLINE),
            218 => Some(Reply::STATSYLINE),
            219 => Some(Reply::ENDOFSTATS),
            221 => Some(Reply::UMODEIS),
            231 => Some(Reply::SERVICEINFO),
            232 => Some(Reply::ENDOFSERVICES),
            233 => Some(Reply::SERVICE),
            234 => Some(Reply::SERVLIST),
            235 => Some(Reply::SERVLISTEND),
            240 => Some(Reply::STATSVLINE),
            241 => Some(Reply::STATSLLINE),
            242 => Some(Reply::STATSUPTIME),
            243 => Some(Reply::STATSOLINE),
            244 => Some(Reply::STATSHLINE),
            246 => Some(Reply::STATSPING),
            247 => Some(Reply::STATSBLINE),
            250 => Some(Reply::STATSDLINE),
            251 => Some(Reply::LUSERCLIENT),
            252 => Some(Reply::LUSEROP),
            253 => Some(Reply::LUSERUNKNOWN),
            254 => Some(Reply::LUSERCHANNELS),
            255 => Some(Reply::LUSERME),
            256 => Some(Reply::ADMINME),
            257 => Some(Reply::ADMINLOC1),
            258 => Some(Reply::ADMINLOC2),
            259 => Some(Reply::ADMINEMAIL),
            261 => Some(Reply::TRACELOG),
            262 => Some(Reply::TRACEEND),
            263 => Some(Reply::TRYAGAIN),
            300 => Some(Reply::NONE),
            301 => Some(Reply::AWAY),
            302 => Some(Reply::USERHOST),
            303 => Some(Reply::ISON),
            305 => Some(Reply::UNAWAY),
            306 => Some(Reply::NOWAWAY),
            311 => Some(Reply::WHOISUSER),
            312 => Some(Reply::WHOISSERVER),
            313 => Some(Reply::WHOISOPERATOR),
            314 => Some(Reply::WHOWASUSER),
            315 => Some(Reply::ENDOFWHO),
            316 => Some(Reply::WHOISCHANOP),
            317 => Some(Reply::WHOISIDLE),
            318 => Some(Reply::ENDOFWHOIS),
            319 => Some(Reply::WHOISCHANNELS),
            321 => Some(Reply::LISTSTART),
            322 => Some(Reply::LIST),
            323 => Some(Reply::LISTEND),
            324 => Some(Reply::CHANNELMODEIS),
            325 => Some(Reply::UNIQOPIS),
            331 => Some(Reply::NOTOPIC),
            332 => Some(Reply::TOPIC),
            341 => Some(Reply::INVITING),
            342 => Some(Reply::SUMMONING),
            346 => Some(Reply::INVITELIST),
            347 => Some(Reply::ENDOFINVITELIST),
            348 => Some(Reply::EXCEPTLIST),
            349 => Some(Reply::ENDOFEXCEPTLIST),
            351 => Some(Reply::VERSION),
            352 => Some(Reply::WHOREPLY),
            353 => Some(Reply::NAMREPLY),
            361 => Some(Reply::KILLDONE),
            362 => Some(Reply::CLOSING),
            363 => Some(Reply::CLOSEEND),
            364 => Some(Reply::LINKS),
            365 => Some(Reply::ENDOFLINKS),
            366 => Some(Reply::ENDOFNAMES),
            367 => Some(Reply::BANLIST),
            368 => Some(Reply::ENDOFBANLIST),
            369 => Some(Reply::ENDOFWHOWAS),
            371 => Some(Reply::INFO),
            372 => Some(Reply::MOTD),
            373 => Some(Reply::INFOSTART),
            374 => Some(Reply::ENDOFINFO),
            375 => Some(Reply::MOTDSTART),
            376 => Some(Reply::ENDOFMOTD),
            381 => Some(Reply::YOUREOPER),
            382 => Some(Reply::REHASHING),
            383 => Some(Reply::YOURESERVICE),
            384 => Some(Reply::MYPORTIS),
            391 => Some(Reply::TIME),
            392 => Some(Reply::USERSSTART),
            393 => Some(Reply::USERS),
            394 => Some(Reply::ENDOFUSERS),
            395 => Some(Reply::NOUSERS),
            _ => None,
        }
    }
}

// FIXME: docs
/// Numeric error returned by the server.
#[allow(missing_docs)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    NOSUCHNICK = 401,
    NOSUCHSERVER = 402,
    NOSUCHCHANNEL = 403,
    CANNOTSENDTOCHAN = 404,
    TOOMANYCHANNELS = 405,
    WASNOSUCHNICK = 406,
    TOOMANYTARGETS = 407,
    NOSUCHSERVICE = 408,
    NOORIGIN = 409,
    NORECIPIENT = 411,
    NOTEXTTOSEND = 412,
    NOTOPLEVEL = 413,
    WILDTOPLEVEL = 414,
    BADMASK = 415,
    UNKNOWNCOMMAND = 421,
    NOMOTD = 422,
    NOADMININFO = 423,
    FILEERROR = 424,
    NONICKNAMEGIVEN = 431,
    ERRONEUSNICKNAME = 432,
    NICKNAMEINUSE = 433,
    NICKCOLLISION = 436,
    UNAVAILRESOURCE = 437,
    USERNOTINCHANNEL = 441,
    NOTONCHANNEL = 442,
    USERONCHANNEL = 443,
    NOLOGIN = 444,
    SUMMONDISABLED = 445,
    USERSDISABLED = 446,
    NOTREGISTERED = 451,
    NEEDMOREPARAMS = 461,
    ALREADYREGISTRED = 462,
    NOPERMFORHOST = 463,
    PASSWDMISMATCH = 464,
    YOUREBANNEDCREEP = 465,
    YOUWILLBEBANNED = 466,
    KEYSET = 467,
    CHANNELISFULL = 471,
    UNKNOWNMODE = 472,
    INVITEONLYCHAN = 473,
    BANNEDFROMCHAN = 474,
    BADCHANNELKEY = 475,
    BADCHANMASK = 476,
    NOCHANMODES = 477,
    BANLISTFULL = 478,
    NOPRIVILEGES = 481,
    CHANOPRIVSNEEDED = 482,
    CANTKILLSERVER = 483,
    RESTRICTED = 484,
    UNIQOPPRIVSNEEDED = 485,
    NOOPERHOST = 491,
    NOSERVICEHOST = 492,
    UMODEUNKNOWNFLAG = 501,
    USERSDONTMATCH = 502,
}

impl Error {
    fn from(data: u16) -> Option<Error> {
        match data {
            401 => Some(Error::NOSUCHNICK),
            402 => Some(Error::NOSUCHSERVER),
            403 => Some(Error::NOSUCHCHANNEL),
            404 => Some(Error::CANNOTSENDTOCHAN),
            405 => Some(Error::TOOMANYCHANNELS),
            406 => Some(Error::WASNOSUCHNICK),
            407 => Some(Error::TOOMANYTARGETS),
            408 => Some(Error::NOSUCHSERVICE),
            409 => Some(Error::NOORIGIN),
            411 => Some(Error::NORECIPIENT),
            412 => Some(Error::NOTEXTTOSEND),
            413 => Some(Error::NOTOPLEVEL),
            414 => Some(Error::WILDTOPLEVEL),
            415 => Some(Error::BADMASK),
            421 => Some(Error::UNKNOWNCOMMAND),
            422 => Some(Error::NOMOTD),
            423 => Some(Error::NOADMININFO),
            424 => Some(Error::FILEERROR),
            431 => Some(Error::NONICKNAMEGIVEN),
            432 => Some(Error::ERRONEUSNICKNAME),
            433 => Some(Error::NICKNAMEINUSE),
            436 => Some(Error::NICKCOLLISION),
            437 => Some(Error::UNAVAILRESOURCE),
            441 => Some(Error::USERNOTINCHANNEL),
            442 => Some(Error::NOTONCHANNEL),
            443 => Some(Error::USERONCHANNEL),
            444 => Some(Error::NOLOGIN),
            445 => Some(Error::SUMMONDISABLED),
            446 => Some(Error::USERSDISABLED),
            451 => Some(Error::NOTREGISTERED),
            461 => Some(Error::NEEDMOREPARAMS),
            462 => Some(Error::ALREADYREGISTRED),
            463 => Some(Error::NOPERMFORHOST),
            464 => Some(Error::PASSWDMISMATCH),
            465 => Some(Error::YOUREBANNEDCREEP),
            466 => Some(Error::YOUWILLBEBANNED),
            467 => Some(Error::KEYSET),
            471 => Some(Error::CHANNELISFULL),
            472 => Some(Error::UNKNOWNMODE),
            473 => Some(Error::INVITEONLYCHAN),
            474 => Some(Error::BANNEDFROMCHAN),
            475 => Some(Error::BADCHANNELKEY),
            476 => Some(Error::BADCHANMASK),
            477 => Some(Error::NOCHANMODES),
            478 => Some(Error::BANLISTFULL),
            481 => Some(Error::NOPRIVILEGES),
            482 => Some(Error::CHANOPRIVSNEEDED),
            483 => Some(Error::CANTKILLSERVER),
            484 => Some(Error::RESTRICTED),
            485 => Some(Error::UNIQOPPRIVSNEEDED),
            491 => Some(Error::NOOPERHOST),
            492 => Some(Error::NOSERVICEHOST),
            501 => Some(Error::UMODEUNKNOWNFLAG),
            502 => Some(Error::USERSDONTMATCH),
            _ => None,
        }
    }
}

// FIXME: docs
#[allow(missing_docs)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KnownCommand {
    PASS,
    NICK,
    USER,
    OPER,
    MODE,
    SERVICE,
    QUIT,
    SQUIT,
    JOIN,
    PART,
    TOPIC,
    NAMES,
    LIST,
    INVITE,
    KICK,
    PRIVMSG,
    NOTICE,
    MOTD,
    LUSERS,
    VERSION,
    STATS,
    LINKS,
    TIME,
    CONNECT,
    TRACE,
    ADMIN,
    INFO,
    SERVLIST,
    SQUERY,
    WHO,
    WHOIS,
    WHOWAS,
    KILL,
    PING,
    PONG,
    ERROR,
    AWAY,
    REHASH,
    DIE,
    RESTART,
    SUMMON,
    USERS,
    WALLOPS,
    USERHOST,
    ISON,
}

impl KnownCommand {
    fn from(data: &[u8]) -> Option<KnownCommand> {
        match data {
            b"PASS" => Some(KnownCommand::PASS),
            b"NICK" => Some(KnownCommand::NICK),
            b"USER" => Some(KnownCommand::USER),
            b"OPER" => Some(KnownCommand::OPER),
            b"MODE" => Some(KnownCommand::MODE),
            b"SERVICE" => Some(KnownCommand::SERVICE),
            b"QUIT" => Some(KnownCommand::QUIT),
            b"SQUIT" => Some(KnownCommand::SQUIT),
            b"JOIN" => Some(KnownCommand::JOIN),
            b"PART" => Some(KnownCommand::PART),
            b"TOPIC" => Some(KnownCommand::TOPIC),
            b"NAMES" => Some(KnownCommand::NAMES),
            b"LIST" => Some(KnownCommand::LIST),
            b"INVITE" => Some(KnownCommand::INVITE),
            b"KICK" => Some(KnownCommand::KICK),
            b"PRIVMSG" => Some(KnownCommand::PRIVMSG),
            b"NOTICE" => Some(KnownCommand::NOTICE),
            b"MOTD" => Some(KnownCommand::MOTD),
            b"LUSERS" => Some(KnownCommand::LUSERS),
            b"VERSION" => Some(KnownCommand::VERSION),
            b"STATS" => Some(KnownCommand::STATS),
            b"LINKS" => Some(KnownCommand::LINKS),
            b"TIME" => Some(KnownCommand::TIME),
            b"CONNECT" => Some(KnownCommand::CONNECT),
            b"TRACE" => Some(KnownCommand::TRACE),
            b"ADMIN" => Some(KnownCommand::ADMIN),
            b"INFO" => Some(KnownCommand::INFO),
            b"SERVLIST" => Some(KnownCommand::SERVLIST),
            b"SQUERY" => Some(KnownCommand::SQUERY),
            b"WHO" => Some(KnownCommand::WHO),
            b"WHOIS" => Some(KnownCommand::WHOIS),
            b"WHOWAS" => Some(KnownCommand::WHOWAS),
            b"KILL" => Some(KnownCommand::KILL),
            b"PING" => Some(KnownCommand::PING),
            b"PONG" => Some(KnownCommand::PONG),
            b"ERROR" => Some(KnownCommand::ERROR),
            b"AWAY" => Some(KnownCommand::AWAY),
            b"REHASH" => Some(KnownCommand::REHASH),
            b"DIE" => Some(KnownCommand::DIE),
            b"RESTART" => Some(KnownCommand::RESTART),
            b"SUMMON" => Some(KnownCommand::SUMMON),
            b"USERS" => Some(KnownCommand::USERS),
            b"WALLOPS" => Some(KnownCommand::WALLOPS),
            b"USERHOST" => Some(KnownCommand::USERHOST),
            b"ISON" => Some(KnownCommand::ISON),
            _ => None,
        }
    }
}

/// Parsed IRC command.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Command<T> {
    /// Numeric reply.
    Reply(Reply),
    /// Numeric error.
    Error(Error),
    /// Command.
    Command(KnownCommand),
    /// An unknown numeric response.
    Numeric(u16),
    /// An unknown string command.
    String(T),
}

/// Parsed IRC message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message<T: ToMut> {
    /// [IRCv3.2 message tags](http://ircv3.net/specs/core/message-tags-3.2.html)
    pub tags: Vec<(T, Option<T::Container>)>,
    /// Message source.
    pub prefix: Prefix<T>,
    /// Command.
    pub command: Command<T>,
    /// Command parameters.
    pub params: Vec<T>,
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
                value: opt!(is_not!(&b"\0\r\n; "[..])),
                || unescape_value(value.unwrap_or(b""))
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

named!(prefix<Prefix<&[u8]> >,
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

fn parse_numeric_response(response: &[u8]) -> Command<&[u8]> {
    let response = ((response[0] - b'0') as u16) * 100 + ((response[1] - b'0') as u16) * 10 + ((response[2] - b'0') as u16);
    if let Some(reply) = Reply::from(response) {
        return Command::Reply(reply);
    }

    if let Some(error) = Error::from(response) {
        return Command::Error(error);
    }

    Command::Numeric(response)
}

fn parse_string_command(cmd: &[u8]) -> Command<&[u8]> {
    if let Some(cmd) = KnownCommand::from(cmd) {
        return Command::Command(cmd);
    }

    Command::String(cmd)
}

named!(command<Command<&[u8]> >,
    alt!(
        map!(digit, parse_numeric_response) |
        map!(alpha, parse_string_command)
    )
);

named!(params<Vec<&[u8]> >,
    chain!(
        params: opt!(
            chain!(
                tag!(b" ") ~
                params: separated_nonempty_list!(tag!(b" "), middle),
                || params
            )
        ) ~
        trailing: opt!(
            chain!(
                tag!(b" :") ~
                trailing: trailing,
                || trailing
            )
        ),
        || {
            let mut params = params.unwrap_or_else(Vec::new);
            params.extend(trailing);
            params
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

named_attr!(#[doc="Parse an IRC message."], pub message<Message<&[u8]> >,
    chain!(
        tags: opt!(tags) ~
        prefix: opt!(prefix) ~
        command: command ~
        params: params ~
        tag!(b"\r\n"),
        || {
            Message {
                tags: tags.unwrap_or_else(Vec::new),
                prefix: prefix.unwrap_or(Prefix::Implicit),
                command: command,
                params: params,
            }
        }
    )
);

/// Example commands and responses from https://dev.twitch.tv/docs/irc/
#[test]
fn twitch_examples() {
    assert_eq!(message(b"PASS oauth:twitch_oauth_token\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::Implicit,
        command: Command::Command(KnownCommand::PASS),
        params: vec![b"oauth:twitch_oauth_token"],
    }));
    assert_eq!(message(b"NICK twitch_username\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::Implicit,
        command: Command::Command(KnownCommand::NICK),
        params: vec![b"twitch_username"],
    }));
    assert_eq!(message(b":tmi.twitch.tv 001 twitch_username :Welcome, GLHF!\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Reply(Reply::WELCOME),
        params: vec![b"twitch_username", b"Welcome, GLHF!"],
    }));
    assert_eq!(message(b":tmi.twitch.tv 002 twitch_username :Your host is tmi.twitch.tv\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Reply(Reply::YOURHOST),
        params: vec![b"twitch_username", b"Your host is tmi.twitch.tv"],
    }));
    assert_eq!(message(b":tmi.twitch.tv 003 twitch_username :This server is rather new\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Reply(Reply::CREATED),
        params: vec![b"twitch_username", b"This server is rather new"],
    }));
    assert_eq!(message(b":tmi.twitch.tv 004 twitch_username :-\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Reply(Reply::MYINFO),
        params: vec![b"twitch_username", b"-"],
    }));
    assert_eq!(message(b":tmi.twitch.tv 375 twitch_username :-\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Reply(Reply::MOTDSTART),
        params: vec![b"twitch_username", b"-"],
    }));
    assert_eq!(message(b":tmi.twitch.tv 372 twitch_username :You are in a maze of twisty passages, all alike.\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Reply(Reply::MOTD),
        params: vec![b"twitch_username", b"You are in a maze of twisty passages, all alike."],
    }));
    assert_eq!(message(b":tmi.twitch.tv 376 twitch_username :>\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Reply(Reply::ENDOFMOTD),
        params: vec![b"twitch_username", b">"],
    }));
    assert_eq!(message(b"WHO #channel\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::Implicit,
        command: Command::Command(KnownCommand::WHO),
        params: vec![b"#channel"],
    }));
    assert_eq!(message(b":tmi.twitch.tv 421 twitch_username WHO :Unknown command\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Error(Error::UNKNOWNCOMMAND),
        params: vec![b"twitch_username", b"WHO", b"Unknown command"],
    }));
    assert_eq!(message(b"JOIN #channel\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::Implicit,
        command: Command::Command(KnownCommand::JOIN),
        params: vec![b"#channel"],
    }));
    assert_eq!(message(b":twitch_username!twitch_username@twitch_username.tmi.twitch.tv JOIN #channel\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::User {
            nick: &b"twitch_username"[..],
            user: Some(b"twitch_username"),
            host: Some(b"twitch_username.tmi.twitch.tv"),
        },
        command: Command::Command(KnownCommand::JOIN),
        params: vec![b"#channel"],
    }));
    assert_eq!(message(b":twitch_username.tmi.twitch.tv 353 twitch_username = #channel :twitch_username\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::Server(b"twitch_username.tmi.twitch.tv"),
        command: Command::Reply(Reply::NAMREPLY),
        params: vec![b"twitch_username", b"=", b"#channel", b"twitch_username"],
    }));
    assert_eq!(message(b":twitch_username.tmi.twitch.tv 366 twitch_username #channel :End of /NAMES list\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::Server(b"twitch_username.tmi.twitch.tv"),
        command: Command::Reply(Reply::ENDOFNAMES),
        params: vec![b"twitch_username", b"#channel", b"End of /NAMES list"],
    }));
    assert_eq!(message(b"PART #channel\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::Implicit,
        command: Command::Command(KnownCommand::PART),
        params: vec![b"#channel"],
    }));
    assert_eq!(message(b":twitch_username!twitch_username@twitch_username.tmi.twitch.tv PART #channel\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::User {
            nick: &b"twitch_username"[..],
            user: Some(b"twitch_username"),
            host: Some(b"twitch_username.tmi.twitch.tv"),
        },
        command: Command::Command(KnownCommand::PART),
        params: vec![b"#channel"],
    }));
    assert_eq!(message(b":twitch_username!twitch_username@twitch_username.tmi.twitch.tv PRIVMSG #channel :message here\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::User {
            nick: &b"twitch_username"[..],
            user: Some(b"twitch_username"),
            host: Some(b"twitch_username.tmi.twitch.tv"),
        },
        command: Command::Command(KnownCommand::PRIVMSG),
        params: vec![b"#channel", b"message here"],
    }));
    assert_eq!(message(b"CAP REQ :twitch.tv/membership\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::Implicit,
        command: Command::String(b"CAP"),
        params: vec![b"REQ", b"twitch.tv/membership"],
    }));
    assert_eq!(message(b":tmi.twitch.tv CAP * ACK :twitch.tv/membership\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::String(b"CAP"),
        params: vec![b"*", b"ACK", b"twitch.tv/membership"],
    }));
    assert_eq!(message(b":twitch_username.tmi.twitch.tv 353 twitch_username = #channel :twitch_username user2 user3\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::Server(b"twitch_username.tmi.twitch.tv"),
        command: Command::Reply(Reply::NAMREPLY),
        params: vec![b"twitch_username", b"=", b"#channel", b"twitch_username user2 user3"],
    }));
    assert_eq!(message(b":twitch_username.tmi.twitch.tv 353 twitch_username = #channel :user5 user6 nicknameN\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::Server(b"twitch_username.tmi.twitch.tv"),
        command: Command::Reply(Reply::NAMREPLY),
        params: vec![b"twitch_username", b"=", b"#channel", b"user5 user6 nicknameN"],
    }));
    assert_eq!(message(b":twitch_username.tmi.twitch.tv 366 twitch_username #channel :End of /NAMES list\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::Server(b"twitch_username.tmi.twitch.tv"),
        command: Command::Reply(Reply::ENDOFNAMES),
        params: vec![b"twitch_username", b"#channel", b"End of /NAMES list"],
    }));
    assert_eq!(message(b":twitch_username!twitch_username@twitch_username.tmi.twitch.tv JOIN #channel\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::User {
            nick: &b"twitch_username"[..],
            user: Some(b"twitch_username"),
            host: Some(b"twitch_username.tmi.twitch.tv"),
        },
        command: Command::Command(KnownCommand::JOIN),
        params: vec![b"#channel"],
    }));
    assert_eq!(message(b":twitch_username!twitch_username@twitch_username.tmi.twitch.tv PART #channel\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::User {
            nick: &b"twitch_username"[..],
            user: Some(b"twitch_username"),
            host: Some(b"twitch_username.tmi.twitch.tv"),
        },
        command: Command::Command(KnownCommand::PART),
        params: vec![b"#channel"],
    }));
    assert_eq!(message(b":jtv MODE #channel +o operator_user\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::Server(b"jtv"),
        command: Command::Command(KnownCommand::MODE),
        params: vec![b"#channel", b"+o", b"operator_user"],
    }));
    assert_eq!(message(b":jtv MODE #channel -o operator_user\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::Server(b"jtv"),
        command: Command::Command(KnownCommand::MODE),
        params: vec![b"#channel", b"-o", b"operator_user"],
    }));
    assert_eq!(message(b"CAP REQ :twitch.tv/commands\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::Implicit,
        command: Command::String(b"CAP"),
        params: vec![b"REQ", b"twitch.tv/commands"],
    }));
    assert_eq!(message(b":tmi.twitch.tv CAP * ACK :twitch.tv/commands\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::String(b"CAP"),
        params: vec![b"*", b"ACK", b"twitch.tv/commands"],
    }));
    assert_eq!(message(b"@msg-id=slow_off :tmi.twitch.tv NOTICE #channel :This room is no longer in slow mode.\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![
            (b"msg-id", Some(Cow::Borrowed(b"slow_off"))),
        ],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::Command(KnownCommand::NOTICE),
        params: vec![b"#channel", b"This room is no longer in slow mode."],
    }));
    assert_eq!(message(b":tmi.twitch.tv HOSTTARGET #hosting_channel :target_channel 99999\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::String(b"HOSTTARGET"),
        params: vec![b"#hosting_channel", b"target_channel 99999"],
    }));
    assert_eq!(message(b":tmi.twitch.tv HOSTTARGET #hosting_channel :- 99999\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::String(b"HOSTTARGET"),
        params: vec![b"#hosting_channel", b"- 99999"],
    }));
    assert_eq!(message(b":tmi.twitch.tv CLEARCHAT #channel :twitch_username\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::String(b"CLEARCHAT"),
        params: vec![b"#channel", b"twitch_username"],
    }));
    assert_eq!(message(b":tmi.twitch.tv CLEARCHAT #channel\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::String(b"CLEARCHAT"),
        params: vec![b"#channel"],
    }));
    assert_eq!(message(b":tmi.twitch.tv USERSTATE #channel\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::String(b"USERSTATE"),
        params: vec![b"#channel"],
    }));
    assert_eq!(message(b":tmi.twitch.tv ROOMSTATE #channel\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::String(b"ROOMSTATE"),
        params: vec![b"#channel"],
    }));
    assert_eq!(message(b":tmi.twitch.tv USERNOTICE #channel :message\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::String(b"USERNOTICE"),
        params: vec![b"#channel", b"message"],
    }));
    assert_eq!(message(b"CAP REQ :twitch.tv/tags\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::Implicit,
        command: Command::String(b"CAP"),
        params: vec![b"REQ", b"twitch.tv/tags"],
    }));
    assert_eq!(message(b":tmi.twitch.tv CAP * ACK :twitch.tv/tags\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::String(b"CAP"),
        params: vec![b"*", b"ACK", b"twitch.tv/tags"],
    }));
    assert_eq!(message(b"@badges=global_mod/1,turbo/1;color=#0D4200;display-name=TWITCH_UserNaME;emotes=25:0-4,12-16/1902:6-10;mod=0;room-id=1337;subscriber=0;turbo=1;user-id=1337;user-type=global_mod :twitch_username!twitch_username@twitch_username.tmi.twitch.tv PRIVMSG #channel :Kappa Keepo Kappa\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![
            (b"badges", Some(Cow::Borrowed(b"global_mod/1,turbo/1"))),
            (b"color", Some(Cow::Borrowed(b"#0D4200"))),
            (b"display-name", Some(Cow::Borrowed(b"TWITCH_UserNaME"))),
            (b"emotes", Some(Cow::Borrowed(b"25:0-4,12-16/1902:6-10"))),
            (b"mod", Some(Cow::Borrowed(b"0"))),
            (b"room-id", Some(Cow::Borrowed(b"1337"))),
            (b"subscriber", Some(Cow::Borrowed(b"0"))),
            (b"turbo", Some(Cow::Borrowed(b"1"))),
            (b"user-id", Some(Cow::Borrowed(b"1337"))),
            (b"user-type", Some(Cow::Borrowed(b"global_mod"))),
        ],
        prefix: Prefix::User {
            nick: &b"twitch_username"[..],
            user: Some(b"twitch_username"),
            host: Some(b"twitch_username.tmi.twitch.tv"),
        },
        command: Command::Command(KnownCommand::PRIVMSG),
        params: vec![b"#channel", b"Kappa Keepo Kappa"],
    }));
    assert_eq!(message(b"@badges=staff/1,bits/1000;bits=100;color=;display-name=TWITCH_UserNaME;emotes=;id=b34ccfc7-4977-403a-8a94-33c6bac34fb8;mod=0;room-id=1337;subscriber=0;turbo=1;user-id=1337;user-type=staff :twitch_username!twitch_username@twitch_username.tmi.twitch.tv PRIVMSG #channel :cheer100\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![
            (b"badges", Some(Cow::Borrowed(b"staff/1,bits/1000"))),
            (b"bits", Some(Cow::Borrowed(b"100"))),
            (b"color", Some(Cow::Borrowed(b""))),
            (b"display-name", Some(Cow::Borrowed(b"TWITCH_UserNaME"))),
            (b"emotes", Some(Cow::Borrowed(b""))),
            (b"id", Some(Cow::Borrowed(b"b34ccfc7-4977-403a-8a94-33c6bac34fb8"))),
            (b"mod", Some(Cow::Borrowed(b"0"))),
            (b"room-id", Some(Cow::Borrowed(b"1337"))),
            (b"subscriber", Some(Cow::Borrowed(b"0"))),
            (b"turbo", Some(Cow::Borrowed(b"1"))),
            (b"user-id", Some(Cow::Borrowed(b"1337"))),
            (b"user-type", Some(Cow::Borrowed(b"staff"))),
        ],
        prefix: Prefix::User {
            nick: &b"twitch_username"[..],
            user: Some(b"twitch_username"),
            host: Some(b"twitch_username.tmi.twitch.tv"),
        },
        command: Command::Command(KnownCommand::PRIVMSG),
        params: vec![b"#channel", b"cheer100"],
    }));
    assert_eq!(message(b"@color=#0D4200;display-name=TWITCH_UserNaME;emote-sets=0,33,50,237,793,2126,3517,4578,5569,9400,10337,12239;mod=1;subscriber=1;turbo=1;user-type=staff :tmi.twitch.tv USERSTATE #channel\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![
            (b"color", Some(Cow::Borrowed(b"#0D4200"))),
            (b"display-name", Some(Cow::Borrowed(b"TWITCH_UserNaME"))),
            (b"emote-sets", Some(Cow::Borrowed(b"0,33,50,237,793,2126,3517,4578,5569,9400,10337,12239"))),
            (b"mod", Some(Cow::Borrowed(b"1"))),
            (b"subscriber", Some(Cow::Borrowed(b"1"))),
            (b"turbo", Some(Cow::Borrowed(b"1"))),
            (b"user-type", Some(Cow::Borrowed(b"staff"))),
        ],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::String(b"USERSTATE"),
        params: vec![b"#channel"],
    }));
    assert_eq!(message(b"@color=#0D4200;display-name=TWITCH_UserNaME;emote-sets=0,33,50,237,793,2126,3517,4578,5569,9400,10337,12239;turbo=0;user-id=1337;user-type=admin :tmi.twitch.tv GLOBALUSERSTATE\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![
            (b"color", Some(Cow::Borrowed(b"#0D4200"))),
            (b"display-name", Some(Cow::Borrowed(b"TWITCH_UserNaME"))),
            (b"emote-sets", Some(Cow::Borrowed(b"0,33,50,237,793,2126,3517,4578,5569,9400,10337,12239"))),
            (b"turbo", Some(Cow::Borrowed(b"0"))),
            (b"user-id", Some(Cow::Borrowed(b"1337"))),
            (b"user-type", Some(Cow::Borrowed(b"admin"))),
        ],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::String(b"GLOBALUSERSTATE"),
        params: vec![],
    }));
    assert_eq!(message(b"@broadcaster-lang=;r9k=0;slow=0;subs-only=0 :tmi.twitch.tv ROOMSTATE #channel\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![
            (b"broadcaster-lang", Some(Cow::Borrowed(b""))),
            (b"r9k", Some(Cow::Borrowed(b"0"))),
            (b"slow", Some(Cow::Borrowed(b"0"))),
            (b"subs-only", Some(Cow::Borrowed(b"0"))),
        ],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::String(b"ROOMSTATE"),
        params: vec![b"#channel"],
    }));
    assert_eq!(message(b"@slow=10 :tmi.twitch.tv ROOMSTATE #channel\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![
            (b"slow", Some(Cow::Borrowed(b"10"))),
        ],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::String(b"ROOMSTATE"),
        params: vec![b"#channel"],
    }));
    assert_eq!(message(b"@badges=staff/1,broadcaster/1,turbo/1;color=#008000;display-name=TWITCH_UserName;emotes=;mod=0;msg-id=resub;msg-param-months=6;room-id=1337;subscriber=1;system-msg=TWITCH_UserName\\shas\\ssubscribed\\sfor\\s6\\smonths!;login=twitch_username;turbo=1;user-id=1337;user-type=staff :tmi.twitch.tv USERNOTICE #channel :Great stream -- keep it up!\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![
            (b"badges", Some(Cow::Borrowed(b"staff/1,broadcaster/1,turbo/1"))),
            (b"color", Some(Cow::Borrowed(b"#008000"))),
            (b"display-name", Some(Cow::Borrowed(b"TWITCH_UserName"))),
            (b"emotes", Some(Cow::Borrowed(b""))),
            (b"mod", Some(Cow::Borrowed(b"0"))),
            (b"msg-id", Some(Cow::Borrowed(b"resub"))),
            (b"msg-param-months", Some(Cow::Borrowed(b"6"))),
            (b"room-id", Some(Cow::Borrowed(b"1337"))),
            (b"subscriber", Some(Cow::Borrowed(b"1"))),
            (b"system-msg", Some(Cow::Borrowed(b"TWITCH_UserName has subscribed for 6 months!"))),
            (b"login", Some(Cow::Borrowed(b"twitch_username"))),
            (b"turbo", Some(Cow::Borrowed(b"1"))),
            (b"user-id", Some(Cow::Borrowed(b"1337"))),
            (b"user-type", Some(Cow::Borrowed(b"staff"))),
        ],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::String(b"USERNOTICE"),
        params: vec![b"#channel", b"Great stream -- keep it up!"],
    }));
    assert_eq!(message(b"@badges=staff/1,broadcaster/1,turbo/1;color=#008000;display-name=TWITCH_UserName;emotes=;mod=0;msg-id=resub;msg-param-months=6;room-id=1337;subscriber=1;system-msg=TWITCH_UserName\\shas\\ssubscribed\\sfor\\s6\\smonths!;login=twitch_username;turbo=1;user-id=1337;user-type=staff :tmi.twitch.tv USERNOTICE #channel\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![
            (b"badges", Some(Cow::Borrowed(b"staff/1,broadcaster/1,turbo/1"))),
            (b"color", Some(Cow::Borrowed(b"#008000"))),
            (b"display-name", Some(Cow::Borrowed(b"TWITCH_UserName"))),
            (b"emotes", Some(Cow::Borrowed(b""))),
            (b"mod", Some(Cow::Borrowed(b"0"))),
            (b"msg-id", Some(Cow::Borrowed(b"resub"))),
            (b"msg-param-months", Some(Cow::Borrowed(b"6"))),
            (b"room-id", Some(Cow::Borrowed(b"1337"))),
            (b"subscriber", Some(Cow::Borrowed(b"1"))),
            (b"system-msg", Some(Cow::Borrowed(b"TWITCH_UserName has subscribed for 6 months!"))),
            (b"login", Some(Cow::Borrowed(b"twitch_username"))),
            (b"turbo", Some(Cow::Borrowed(b"1"))),
            (b"user-id", Some(Cow::Borrowed(b"1337"))),
            (b"user-type", Some(Cow::Borrowed(b"staff"))),
        ],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::String(b"USERNOTICE"),
        params: vec![b"#channel"],
    }));
    assert_eq!(message(b"@ban-duration=1;ban-reason=Follow\\sthe\\srules :tmi.twitch.tv CLEARCHAT #channel :target_username\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![
            (b"ban-duration", Some(Cow::Borrowed(b"1"))),
            (b"ban-reason", Some(Cow::Borrowed(b"Follow the rules"))),
        ],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::String(b"CLEARCHAT"),
        params: vec![b"#channel", b"target_username"],
    }));
    assert_eq!(message(b"@ban-reason=Follow\\sthe\\srules :tmi.twitch.tv CLEARCHAT #channel :target_username\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![
            (b"ban-reason", Some(Cow::Borrowed(b"Follow the rules"))),
        ],
        prefix: Prefix::Server(b"tmi.twitch.tv"),
        command: Command::String(b"CLEARCHAT"),
        params: vec![b"#channel", b"target_username"],
    }));
    assert_eq!(message(b"PING :tmi.twitch.tv\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::Implicit,
        command: Command::Command(KnownCommand::PING),
        params: vec![b"tmi.twitch.tv"],
    }));
}

/// Examples from http://ircv3.net/specs/core/message-tags-3.2.html
#[test]
fn ircv32_message_tags_examples() {
    assert_eq!(message(b":nick!ident@host.com PRIVMSG me :Hello\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::User {
            nick: &b"nick"[..],
            user: Some(b"ident"),
            host: Some(b"host.com"),
        },
        command: Command::Command(KnownCommand::PRIVMSG),
        params: vec![b"me", b"Hello"],
    }));
    assert_eq!(message(b"@aaa=bbb;ccc;example.com/ddd=eee :nick!ident@host.com PRIVMSG me :Hello\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![
            (b"aaa", Some(Cow::Borrowed(b"bbb"))),
            (b"ccc", None),
            (b"example.com/ddd", Some(Cow::Borrowed(b"eee"))),
        ],
        prefix: Prefix::User {
            nick: &b"nick"[..],
            user: Some(b"ident"),
            host: Some(b"host.com"),
        },
        command: Command::Command(KnownCommand::PRIVMSG),
        params: vec![b"me", b"Hello"],
    }));
}

/// Things that Twitch does differently.
#[test]
fn twitch_pls() {
    // Nickname starting with a digit.
    assert_eq!(message(b":3and4fifths!3and4fifths@3and4fifths.tmi.twitch.tv PRIVMSG #loadingreadyrun :You missed a window to climb through\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::User {
            nick: &b"3and4fifths"[..],
            user: Some(b"3and4fifths"),
            host: Some(b"3and4fifths.tmi.twitch.tv"),
        },
        command: Command::Command(KnownCommand::PRIVMSG),
        params: vec![b"#loadingreadyrun", b"You missed a window to climb through"],
    }));

    // Hostname component ending with an underscore.
    assert_eq!(message(b":featherweight_!featherweight_@featherweight_.tmi.twitch.tv PRIVMSG #loadingreadyrun :Hello human people\r\n"), nom::IResult::Done(&b""[..], Message::<&[u8]> {
        tags: vec![],
        prefix: Prefix::User {
            nick: &b"featherweight_"[..],
            user: Some(b"featherweight_"),
            host: Some(b"featherweight_.tmi.twitch.tv"),
        },
        command: Command::Command(KnownCommand::PRIVMSG),
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
