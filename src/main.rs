mod handler;

use std::sync::Arc;

use libgen::Utils;
use teloxide::{prelude2::*, dispatching2::UpdateFilterExt};
use handler::{message_handler, callback_handler};

#[tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    teloxide::enable_logging!();
    log::info!("Starting libgen-bot");

    let bot = Bot::from_env().auto_send();
    let utils = Arc::new(Utils::new());

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler))
        .branch(Update::filter_callback_query().endpoint(callback_handler));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![utils])
        .build()
        .setup_ctrlc_handler()
        .dispatch()
        .await;

    log::info!("Closing bot... Goodbye!");
}