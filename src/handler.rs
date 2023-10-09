use crate::libgen::{get_books, get_ids, types::*, Utils};
use crate::utils::*;
use std::{error::Error, sync::Arc};
use teloxide::payloads::EditMessageTextSetters;
use teloxide::types::ParseMode;
use teloxide::{prelude::*, utils::command::BotCommands};

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
    let (user_id, chat_id) = match q.message {
        Some(Message { id, chat, .. }) => (id, chat.id),
        None => return Ok(()),
    };

    let ids = match q.data {
        Some(id) => vec![id.parse().unwrap()],
        None => {
            bot.edit_message_text(chat_id, user_id, "💥").await?;
            return Ok(());
        }
    };

    let book = match get_ids(&utils.client, ids).await {
        Ok(mut books) => books.remove(0),
        Err(_) => {
            bot.edit_message_text(chat_id, user_id, "💥").await?;
            return Ok(());
        }
    };

    utils.register(chat_id.0, user_id.0, "SELECTION")?;

    let url_keyboard = make_url_keyboard(&book.md5_url());
    bot.edit_message_text(chat_id, user_id, book.pretty())
        .parse_mode(ParseMode::Html)
        .reply_markup(url_keyboard)
        .await?;

    Ok(())
}

pub async fn message_handler(
    bot: Bot,
    m: Message,
    utils: Arc<Utils>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let chat_id = m.chat.id;

    let text = match m.text() {
        Some(text) => text.trim(),
        None => return Ok(()),
    };

    let msg = bot.send_message(chat_id, "🤖 Loading...").await?;
    utils.register(chat_id.0, msg.id.0, "INVOKE")?;

    let command = Command::parse(text, "libgenis_bot");
    let mut query = Search::Default(text.into());
    if let Ok(command) = command {
        query = command.into();
    }

    let books = match get_books(&utils.client, query, 5).await {
        Ok(books) => books,
        Err(_) => {
            utils.register(chat_id.0, msg.id.0, "BAD")?;
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
        utils.register(chat_id.0, msg.id.0, "UNAVAILABLE")?;
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

