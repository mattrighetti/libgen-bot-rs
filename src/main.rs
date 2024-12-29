mod db;
mod handler;
mod libgen;
mod utils;

use handler::{callback_handler, message_handler};
use libgen::Utils;
use std::{env, sync::Arc};
use teloxide::prelude::*;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting libgen-bot");

    let bot = Bot::from_env();
    let db_path = env::var("DB_URL").expect("cannot find DB_URL.");
    let utils = Arc::new(Utils::new(db_path).await);

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
