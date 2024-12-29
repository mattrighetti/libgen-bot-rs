use std::{error::Error, sync::Arc};
use teloxide::payloads::EditMessageTextSetters;
use teloxide::types::{MaybeInaccessibleMessage, ParseMode};
use teloxide::{prelude::*, utils::command::BotCommands};

use crate::libgen::{types::*, Utils};
use crate::{db, utils::*};

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    Isbn(String),
    Title(String),
    Author(String),
}

impl From<Command> for Search {
    fn from(command: Command) -> Self {
        match command {
            Command::Author(author) => Search::Author(author),
            Command::Title(title) => Search::Title(title),
            Command::Isbn(isbn) => Search::Isbn(isbn),
        }
    }
}

pub async fn callback_handler(
    q: CallbackQuery,
    bot: Bot,
    utils: Arc<Utils>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let (message_id, chat_id) = match q.message {
        Some(MaybeInaccessibleMessage::Inaccessible(ref msg)) => (msg.message_id, msg.chat.id),
        Some(MaybeInaccessibleMessage::Regular(ref msg)) => (msg.id, msg.chat.id),
        None => return Ok(()),
    };

    let ids = match q.data {
        Some(id) => vec![id.parse().unwrap()],
        None => {
            bot.edit_message_text(chat_id, message_id, "💥").await?;
            return Ok(());
        }
    };

    let book = match utils.client.get_ids(ids).await {
        Ok(mut books) => books.remove(0),
        Err(_) => {
            bot.edit_message_text(chat_id, message_id, "💥").await?;
            return Ok(());
        }
    };

    db::register(&utils.db, chat_id.0, message_id.0, "SELECTION").await?;

    let url_keyboard = make_url_keyboard(&book.md5_url());
    bot.edit_message_text(chat_id, message_id, book.pretty())
        .parse_mode(ParseMode::Html)
        .reply_markup(url_keyboard)
        .await?;

    Ok(())
}

pub async fn message_handler(
    bot: Bot,
    msg: Message,
    utils: Arc<Utils>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let chat_id = msg.chat.id;

    let text = match msg.text() {
        Some(text) => text.trim(),
        None => return Ok(()),
    };

    let msg = bot.send_message(chat_id, "🤖 Loading...").await?;
    db::register(&utils.db, chat_id.0, msg.id.0, "INVOKE").await?;

    let command = Command::parse(text, "libgenis_bot");
    let mut query = Search::Default(text.into());
    if let Ok(command) = command {
        query = command.into();
    }

    let books = match utils.client.get_books(query, 5).await {
        Ok(books) => books,
        Err(_) => {
            db::register(&utils.db, chat_id.0, msg.id.0, "BAD").await?;
            bot.edit_message_text(
                chat_id,
                msg.id,
                "Mmm, something went bad while searching for books. Try again later...",
            )
            .await?;
            return Ok(());
        }
    };

    if books.is_empty() {
        db::register(&utils.db, chat_id.0, msg.id.0, "UNAVAILABLE").await?;
        bot.edit_message_text(
            chat_id,
            msg.id,
            "Sorry, I don't have any result for that...",
        )
        .await?;
    } else {
        let keyboard = make_keyboard(&books);
        let text = make_message(&books);
        bot.edit_message_text(chat_id, msg.id, text)
            .parse_mode(ParseMode::Html)
            .reply_markup(keyboard)
            .await?;
    }

    Ok(())
}
