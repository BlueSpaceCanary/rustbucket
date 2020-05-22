extern crate failure;
extern crate irc;
extern crate openssl_probe;
extern crate ratelimit;
extern crate tokio;

#[macro_use]
extern crate diesel;
extern crate dotenv;

#[macro_use]
extern crate log;
extern crate env_logger;

mod brain;
mod models;
mod schema;

use brain::IdMemory;
use brain::Superego;
use futures::*;
use irc::client::prelude::*;
use std::env;

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    env_logger::init();

    // Needed to make sure openssl works in alpine :/
    openssl_probe::init_ssl_cert_env_vars();

    let args: Vec<String> = env::args().collect();
    let config = if let Some(config_path) = args.get(1) {
        Config::load(config_path)?
    } else {
        Config {
            nickname: Some("awoo".to_owned()),
            server: Some("irc.qrimes.club".to_owned()),
            channels: vec!["#funposting".to_owned()],
            use_tls: Some(true),
            port: Some(6697),
            ..Config::default()
        }
    };

    loop {
        // Clone config so we can restart on error
        let mut client = Client::from_config(config.clone()).await?;
        let mut stream = client.stream()?;
        client.identify()?;

        let mut brain = Superego::<IdMemory>::new(client.current_nickname().to_string());

        // TODO(bluespacecanary) this sucks ass, implement
        // while_match! and become a real rust programmer
        loop {
            match stream.next().await.transpose() {
                Ok(Some(msg)) => {
                    if let Command::PRIVMSG(channel, msg) = msg.command {
                        if let Some(resp) = brain.respond(&msg.to_string()) {
                            match client.send_privmsg(&channel, resp.clone()) {
                                Ok(()) => info!("responded with {}", resp), // keep matching messages
                                Err(e) => {
                                    error!("Died while sending message: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                }
                Ok(None) => warn!("Got an empty message"),
                Err(e) => {
                    error!("Message stream died: {}; attempting to reconnect", e);
                    break;
                }
            }
        }
    }
}
