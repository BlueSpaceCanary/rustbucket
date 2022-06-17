extern crate anyhow;
extern crate irc;
extern crate openssl_probe;
extern crate tokio;

extern crate governor;
extern crate nonzero_ext;

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

use log::Level;
use std::{env, time::Duration};

use governor::{Quota, RateLimiter};
use nonzero_ext::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // init logging
    env_logger::init();

    // metrics
    let receiver = Receiver::builder()
        .build()
        .expect("failed to create receiver");

    let mut sink = receiver.sink();

    // Needed to make sure openssl works in alpine :/
    openssl_probe::init_ssl_cert_env_vars();

    let args: Vec<String> = env::args().collect();
    let config = if let Some(config_path) = args.get(1) {
        Config::load(config_path)?
    } else {
        panic!("No config provided")
    };

    loop {
        // Clone config so we can restart on error
        let mut client = Client::from_config(config.clone()).await?;
        let mut stream = client.stream()?;
        client.identify()?;

        let lim = RateLimiter::keyed(Quota::per_minute(nonzero!(6u32)));
        let mut brain = Superego::<IdMemory>::new(client.current_nickname().to_string());

        // TODO(bluespacecanary) this sucks ass, implement
        // while_match! and become a real rust programmer
        loop {
            match stream.next().await.transpose() {
                Ok(Some(msg)) => {
                    if let Some(lim_nick) = msg.source_nickname() {
                        let lim_nick = lim_nick.to_owned();
                        if let Command::PRIVMSG(channel, msg) = msg.command {
                            if let Ok(()) = lim.check_key(&lim_nick) {
                                if let Some(resp) = brain.respond(&msg.to_string()) {
                                    if let Err(e) = match client.send_privmsg(&channel, resp.clone()) {
                                        error!("Died while sending message: {}", e);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(None) => {
                    info!("Empty message")
                }
                Err(e) => {
                    error!("Message stream died: {}; attempting to reconnect", e);
                    break;
                }
            }
        }

		warning!("Restarting responder loop")
    }
}
