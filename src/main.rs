extern crate irc;
extern crate openssl_probe;
extern crate ratelimit;

mod factoid;

use crate::factoid::Brain;
use irc::client::prelude::*;
use std::env;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
    // Needed to make sure openssl works in alpine :/
    openssl_probe::init_ssl_cert_env_vars();

    let args: Vec<String> = env::args().collect();

    let config = if let Some(config_path) = args.get(1) {
        match Config::load(config_path) {
            Ok(conf) => conf,
            Err(e) => panic!("error parsing config: {:?}", e),
        }
    } else {
        Config {
            nickname: Some("awoo".to_owned()),
            server: Some("irc.qrimes.club".to_owned()),
            channels: Some(vec!["#funposting".to_owned()]),
            use_ssl: Some(true),
            port: Some(6697),
            ..Config::default()
        }
    };

    let cns = Arc::new(Mutex::new(Brain::new(config.nickname.clone().unwrap())));

    let mut reactor = IrcReactor::new().unwrap();
    let client = reactor.prepare_client_and_connect(&config).unwrap();
    client.identify().unwrap();

    reactor.register_client_with_handler(client, move |client, message| {
        let cns = Arc::clone(&cns);
        let mut brain = cns.lock().unwrap();
        connection_handler(config.clone(), client, message, &mut *brain);
        Ok(())
    });

    loop {
        match reactor.run() {
            Ok(()) => continue,
            Err(irc::error::IrcError::PingTimeout) => continue,
            Err(e) => panic!("{:?}", e),
        }
    }
}

fn connection_handler(
    _config: Config,
    client: &IrcClient,
    message: irc::proto::Message,
    brain: &mut Brain,
) {
    // And here we can do whatever we want with the messages.
    if let Command::PRIVMSG(ref target, ref msg) = message.command {
        println!("{}", msg);
        if let Some(resp) = brain.respond(msg) {
            client
                .send_privmsg(message.response_target().unwrap_or(target), resp)
                .unwrap();
        }
    }
}
