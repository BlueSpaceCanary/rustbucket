extern crate irc;
extern crate ratelimit;

mod factoid;

use crate::factoid::Brain;
use irc::client::prelude::*;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
    // We can also load the Config at runtime via Config::load("path/to/config.toml")
    let config = Config {
        nickname: Some("awoo".to_owned()),
        server: Some("irc.qrimes.club".to_owned()),
        channels: Some(vec!["#funposting".to_owned()]),
        use_ssl: Some(true),
        port: Some(6697),
        ..Config::default()
    };

    let cns = Arc::new(Mutex::new(Brain::new(config.nickname.clone().unwrap())));

    let mut reactor = IrcReactor::new().unwrap();
    let client = reactor.prepare_client_and_connect(&config).unwrap();
    client.identify().unwrap();

    let mut ratelimit = ratelimit::Builder::new()
        .capacity(10) //number of tokens the bucket will hold
        .quantum(5) //add five tokens per interval
        .interval(Duration::new(30, 0)) //add quantum tokens every 30 seconds
        .build();

    let mut handle = ratelimit.make_handle();
    thread::spawn(move || ratelimit.run());

    reactor.register_client_with_handler(client, move |client, message| {
        let cns = Arc::clone(&cns);
        let mut brain = cns.lock().unwrap();
        connection_handler(config.clone(), client, message, &mut *brain, &mut handle);
        Ok(())
    });

    reactor.run().unwrap();
}

fn connection_handler(
    _config: Config,
    client: &IrcClient,
    message: irc::proto::Message,
    brain: &mut Brain,
    handle: &mut ratelimit::Handle,
) {
    // And here we can do whatever we want with the messages.
    if let Command::PRIVMSG(ref target, ref msg) = message.command {
        println!("{}", msg);
        match handle.try_wait() {
            Ok(()) => {
                match brain.respond(msg) {
                    Some(s) => client
                        .send_privmsg(message.response_target().unwrap_or(target), s)
                        .unwrap(),
                    _ => (),
                };
            }
            _ => (), // If out of tokens, just skip it
        }
    }
}
