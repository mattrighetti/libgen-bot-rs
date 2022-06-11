use std::{
    error::Error,
    sync::Arc
};
use crate::libgen::{
    types::*,
    Utils,
    get_ids,
    get_books
};
use crate::utils::*;
use teloxide::payloads::EditMessageTextSetters;
use teloxide::types::ParseMode;
use teloxide::{
    prelude2::*,
    Bot,
    adaptors::AutoSend,
    types::{
        Message,
    },
    utils::command::BotCommand 
};

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    ISBN(String),
    Title(String),
    Author(String)
}

impl From<Command> for Search {
    fn from(command: Command) -> Self {
        match command {
            Command::Author(author) => Search::Author(author),
            Command::Title(title) => Search::Title(title),
            Command::ISBN(isbn) => Search::ISBN(isbn)
        } 
    }
}

pub async fn callback_handler(
    q: CallbackQuery,
    bot: AutoSend<Bot>,
    utils: Arc<Utils>
) 
    -> Result<(), Box<dyn Error + Send + Sync>> 
{
    if let Some(id) = q.data {
        let ids = vec![id.parse().unwrap()];
        let books = get_ids(&utils.client, ids).await;
        let book = books.first().unwrap();
        let url_keyboard = make_url_keyboard(&book.md5_url());
        
        match q.message {
            Some(Message { id, chat, .. }) => {
                log::info!("{} selected: {}", chat.id, book.md5);
                bot.edit_message_text(chat.id, id, book.pretty())
                    .parse_mode(ParseMode::Html)
                    .reply_markup(url_keyboard)
                    .await?;
            }
            None => {
                if let Some(id) = q.inline_message_id {
                    bot.edit_message_text_inline(id, "".to_string()).await?;
                }
            }
        }
    }

    Ok(())
}

pub async fn message_handler(
    bot: AutoSend<Bot>,
    m: Message,
    utils: Arc<Utils>
)
    -> Result<(), Box<dyn Error + Send + Sync>> 
{
    let chat_id = m.chat_id();

    let text = match m.text() {
        Some(text) => text.trim(),
        None => { 
            return Ok(()); 
        }
    };

    log::info!("{} contacted bot: {}", chat_id, text);
    let msg = bot.send_message(chat_id, "🤖 Loading...").await?;
    let command =  Command::parse(text, "libgenis_bot");
    let mut q = Search::Default(text.into());
    if let Ok(command) = command {
        q = command.into();
    }

    let books = get_books(&utils.client, q, 5).await;
    if books.len() > 0 {
        let keyboard = make_keyboard(&books);
        let text = make_message(&books);
        bot.edit_message_text(chat_id, msg.id, text)
            .parse_mode(ParseMode::Html)
            .reply_markup(keyboard)
            .await?;
    } else {
        bot.edit_message_text(chat_id, msg.id, "Sorry, I don't have any result for that...").await?;
    }

    Ok(())
}