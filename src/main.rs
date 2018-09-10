extern crate irc;
mod factoid;

use irc::client::prelude::*;

fn main() {
    // We can also load the Config at runtime via Config::load("path/to/config.toml")
    let config = Config {
        nickname: Some("sidra2".to_owned()),
        server: Some("irc.crimes.international".to_owned()),
        channels: Some(vec!["#test".to_owned()]),
        ..Config::default()
    };

    let mut reactor = IrcReactor::new().unwrap();
    let client = reactor.prepare_client_and_connect(&config).unwrap();
    client.identify().unwrap();

    reactor.register_client_with_handler(client, |client, message| {
        print!("{}", message);
        // And here we can do whatever we want with the messages.
        Ok(())
    });

    reactor.run().unwrap();
}
