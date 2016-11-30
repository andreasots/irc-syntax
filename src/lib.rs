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
    Implicit,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Message<'a> {
    pub tags: Vec<(&'a [u8], &'a [u8])>,
    pub prefix: Prefix<'a>,
    pub command: &'a [u8],
    pub params: Vec<&'a [u8]>,
}

named!(prefix<Prefix>,
    map!(
        opt!(
            chain!(
                tag!(&b":"[..]) ~
                prefix: take_until!(&b" "[..]),
                || Prefix::Server(prefix)
            )
        ),
        |prefix| match prefix {
            Some(prefix) => prefix,
            None => Prefix::Implicit,
        }
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
        prefix: prefix ~
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

    /// Example commands and responses from https://dev.twitch.tv/docs/irc/
    #[test]
    fn twitch_examples() {
        assert_eq!(message(b"PASS oauth:twitch_oauth_token\r\n"), Done(&b""[..], Message {
            tags: vec![],
            prefix: Prefix::Implicit,
            command: b"PASS",
            params: vec![b"oauth:twitch_oauth_token"],
        }));
        assert_eq!(message(b"NICK twitch_username\r\n"), Done(&b""[..], Message {
            tags: vec![],
            prefix: Prefix::Implicit,
            command: b"NICK",
            params: vec![b"twitch_username"],
        }));
        assert_eq!(message(b":tmi.twitch.tv 001 twitch_username :Welcome, GLHF!\r\n"), Done(&b""[..], Message {
            tags: vec![],
            prefix: Prefix::Server(b"tmi.twitch.tv"),
            command: b"001",
            params: vec![b"twitch_username", b"Welcome, GLHF!"],
        }));
        assert_eq!(message(b":tmi.twitch.tv 002 twitch_username :Your host is tmi.twitch.tv\r\n"), Done(&b""[..], Message {
            tags: vec![],
            prefix: Prefix::Server(b"tmi.twitch.tv"),
            command: b"002",
            params: vec![b"twitch_username", b"Your host is tmi.twitch.tv"],
        }));
        /*
:tmi.twitch.tv 003 twitch_username :This server is rather new
:tmi.twitch.tv 004 twitch_username :-
:tmi.twitch.tv 375 twitch_username :-
:tmi.twitch.tv 372 twitch_username :You are in a maze of twisty passages, all alike.
:tmi.twitch.tv 376 twitch_username :>
WHO #channel
:tmi.twitch.tv 421 twitch_username WHO :Unknown command
JOIN #channel
:twitch_username!twitch_username@twitch_username.tmi.twitch.tv JOIN #channel
:twitch_username.tmi.twitch.tv 353 twitch_username = #channel :twitch_username
:twitch_username.tmi.twitch.tv 366 twitch_username #channel :End of /NAMES list
PART #channel
:twitch_username!twitch_username@twitch_username.tmi.twitch.tv PART #channel
:twitch_username!twitch_username@twitch_username.tmi.twitch.tv PRIVMSG #channel :message here
CAP REQ :twitch.tv/membership
:tmi.twitch.tv CAP * ACK :twitch.tv/membership
:twitch_username.tmi.twitch.tv 353 twitch_username = #channel :twitch_username user2 user3
:twitch_username.tmi.twitch.tv 353 twitch_username = #channel :user5 user6 nicknameN
:twitch_username.tmi.twitch.tv 366 twitch_username #channel :End of /NAMES list
:twitch_username!twitch_username@twitch_username.tmi.twitch.tv JOIN #channel
:twitch_username!twitch_username@twitch_username.tmi.twitch.tv PART #channel
:jtv MODE #channel +o operator_user
:jtv MODE #channel -o operator_user
CAP REQ :twitch.tv/commands
:tmi.twitch.tv CAP * ACK :twitch.tv/commands
@msg-id=slow_off :tmi.twitch.tv NOTICE #channel :This room is no longer in slow mode.
:tmi.twitch.tv HOSTTARGET #hosting_channel :target_channel [number]
:tmi.twitch.tv HOSTTARGET #hosting_channel :- [number]
:tmi.twitch.tv CLEARCHAT #channel :twitch_username
:tmi.twitch.tv CLEARCHAT #channel
:tmi.twitch.tv USERSTATE #channel
:tmi.twitch.tv ROOMSTATE #channel
:tmi.twitch.tv USERNOTICE #channel :message
CAP REQ :twitch.tv/tags
:tmi.twitch.tv CAP * ACK :twitch.tv/tags
@badges=global_mod/1,turbo/1;color=#0D4200;display-name=TWITCH_UserNaME;emotes=25:0-4,12-16/1902:6-10;mod=0;room-id=1337;subscriber=0;turbo=1;user-id=1337;user-type=global_mod :twitch_username!twitch_username@twitch_username.tmi.twitch.tv PRIVMSG #channel :Kappa Keepo Kappa
@badges=staff/1,bits/1000;bits=100;color=;display-name=TWITCH_UserNaME;emotes=;id=b34ccfc7-4977-403a-8a94-33c6bac34fb8;mod=0;room-id=1337;subscriber=0;turbo=1;user-id=1337;user-type=staff :twitch_username!twitch_username@twitch_username.tmi.twitch.tv PRIVMSG #channel :cheer100
@color=#0D4200;display-name=TWITCH_UserNaME;emote-sets=0,33,50,237,793,2126,3517,4578,5569,9400,10337,12239;mod=1;subscriber=1;turbo=1;user-type=staff :tmi.twitch.tv USERSTATE #channel
@color=#0D4200;display-name=TWITCH_UserNaME;emote-sets=0,33,50,237,793,2126,3517,4578,5569,9400,10337,12239;turbo=0;user-id=1337;user-type=admin :tmi.twitch.tv GLOBALUSERSTATE
@broadcaster-lang=;r9k=0;slow=0;subs-only=0 :tmi.twitch.tv ROOMSTATE #channel
@slow=10 :tmi.twitch.tv ROOMSTATE #channel
@badges=staff/1,broadcaster/1,turbo/1;color=#008000;display-name=TWITCH_UserName;emotes=;mod=0;msg-id=resub;msg-param-months=6;room-id=1337;subscriber=1;system-msg=TWITCH_UserName\shas\ssubscribed\sfor\s6\smonths!;login=twitch_username;turbo=1;user-id=1337;user-type=staff :tmi.twitch.tv USERNOTICE #channel :Great stream -- keep it up!
@badges=staff/1,broadcaster/1,turbo/1;color=#008000;display-name=TWITCH_UserName;emotes=;mod=0;msg-id=resub;msg-param-months=6;room-id=1337;subscriber=1;system-msg=TWITCH_UserName\shas\ssubscribed\sfor\s6\smonths!;login=twitch_username;turbo=1;user-id=1337;user-type=staff :tmi.twitch.tv USERNOTICE #channel
@ban-duration=1;ban-reason=Follow\sthe\srules :tmi.twitch.tv CLEARCHAT #channel :target_username
@ban-reason=Follow\sthe\srules :tmi.twitch.tv CLEARCHAT #channel :target_username
*/
    }
}
