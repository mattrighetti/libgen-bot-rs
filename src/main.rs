mod db;
mod handler;
mod libgen;
mod utils;

use handler::{callback_handler, message_handler};
use libgen::Utils;
use log4rs::config::RawConfig;
use std::{env, sync::Arc};
use teloxide::prelude::*;

const LOG_CONFIG: &str = include_str!("../log.yml");

#[tokio::main]
async fn main() {
    if let Ok(log_path) = std::env::var("LOG_PATH") {
        log4rs::init_file(log_path, Default::default()).unwrap();
    } else {
        let raw_config: RawConfig = serde_yaml::from_str(LOG_CONFIG).unwrap();
        log4rs::init_raw_config(raw_config).unwrap();
    }
    run().await;
}

async fn run() {
    log::info!("Starting libgen-bot");

    let bot = Bot::from_env();
    let db_path = env::var("DB_PATH").unwrap_or("db.sqlite".into());
    let utils = Arc::new(Utils::new(db_path));

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler))
        .branch(Update::filter_callback_query().endpoint(callback_handler));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![utils])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    log::info!("Closing bot... Goodbye!");
}
