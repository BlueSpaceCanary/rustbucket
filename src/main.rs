extern crate failure;
extern crate irc;
extern crate openssl_probe;
extern crate tokio;

extern crate governor;
extern crate nonzero_ext;

extern crate metrics;
extern crate metrics_runtime;

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
use metrics::counter;
use metrics_runtime::{exporters::LogExporter, observers::YamlBuilder, Receiver};
use std::{env, time::Duration};

use governor::{Quota, RateLimiter};
use nonzero_ext::*;

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    // init logging
    env_logger::init();

    // metrics
    let receiver = Receiver::builder()
        .build()
        .expect("failed to create receiver")
		.install();

    // export metrics
    let exporter = LogExporter::new(
        receiver.controller(),
        YamlBuilder::new(),
        Level::Info,
        Duration::from_secs(30),
    );

    tokio::spawn(exporter.async_run());
    receiver.install();

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
                Ok(Some(msg)) => msg.source_nickname().and_then(|lim_nick| {
                    let lim_nick = lim_nick.to_owned();
                    if let Command::PRIVMSG(channel, msg) = msg.command {
                        if let Err(e) = lim
                            .check_key(&lim_nick)
                            .and_then(|x| brain.respond(&x.to_string()))
                            .and_then(|x| client.send_privmsg(&channel, x.clone()))
                            .and_then(|x| counter!("responses_sent", 1))
                        {
                            counter!("failed_response_sends", 1);
                            error!("Died while sending message: {}", e);
                            break;
                        }
                    }
                }),

                Ok(None) => {
                    counter!("empty_messages", 1);
                }
                Err(e) => {
                    counter!("stream_disconnects", 1);
                    error!("Message stream died: {}; attempting to reconnect", e);
                    break;
                }
            }

            counter!("restarts", 1);
        }
    }
}
