extern crate irc;
extern crate rand;

mod factoid;

use factoid::Brain;
use factoid::FactoidKnowledge;
use irc::client::prelude::*;
use std::sync::{Arc, Mutex};

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

    reactor.register_client_with_handler(client, move |client, message| {
        let cns = Arc::clone(&cns);
        let mut brain = cns.lock().unwrap();

        connection_handler(config.clone(), client, message, &mut *brain);
        Ok(())
    });

    reactor.run().unwrap();
}

fn connection_handler(
    config: Config,
    client: &IrcClient,
    message: irc::proto::Message,
    brain: &mut Brain,
) {
    let name = config.nickname.clone().unwrap();

    // And here we can do whatever we want with the messages.
    if let Command::PRIVMSG(ref target, ref msg) = message.command {
        println!("{}", msg);
        if factoid::is_awoo(msg) {
            client
                .send_privmsg(message.response_target().unwrap_or(target), msg)
                .unwrap();
        } else if factoid::is_meow(msg) {
            client
                .send_privmsg(message.response_target().unwrap_or(target), msg)
                .unwrap();
        } else if factoid::creates_factoid(&name, &msg) {
            match brain.create_factoid(msg.to_string()) {
                Ok(_) => {
                    println!("Added fact!");
                    client
                        .send_privmsg(message.response_target().unwrap_or(target), &"OK!")
                        .unwrap();
                }
                Err(e) => panic!("AHHHH"),
            }
        } else if let Some(v) = brain.get_factoid(&msg) {
            println!("hi!");
            client
                .send_privmsg(message.response_target().unwrap_or(target), &v)
                .unwrap();
        }
    }
}
