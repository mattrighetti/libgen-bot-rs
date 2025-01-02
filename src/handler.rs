use std::convert::TryFrom;
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
    #[command(description = "check if I'm alive.")]
    Start,
    Help,
    #[command(description = "search by isbn")]
    Isbn(String),
    #[command(description = "search by title")]
    Title(String),
    #[command(description = "search by author")]
    Author(String),
}

impl TryFrom<Command> for Search {
    type Error = String;

    fn try_from(cmd: Command) -> Result<Self, Self::Error> {
        match cmd {
            Command::Author(author) => Ok(Search::Author(author)),
            Command::Title(title) => Ok(Search::Title(title)),
            Command::Isbn(isbn) => Ok(Search::Isbn(isbn)),
            _ => Err("cannot convert command to search function".into()),
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

    let command = Command::parse(text, "libgenis_bot");

    match command {
        Ok(Command::Start) => {
            db::register(&utils.db, chat_id.0, msg.id.0, "START").await?;
            bot.send_message(chat_id, r"This bot simply queries library genesis for the books that you ask for.
                You will still need to go to library Genesis to download the book.
                If your browser can't open the download link provided to you it's probably because your ISP or local authorities have blocked that url.
                "
            ).await?;

            return Ok(());
        }
        Ok(Command::Help) => {
            db::register(&utils.db, chat_id.0, msg.id.0, "HELP").await?;
            bot.send_message(chat_id, Command::descriptions().to_string())
                .await?;

            return Ok(());
        }
        _ => {}
    }

    let msg = bot.send_message(chat_id, "🤖 Loading...").await?;
    db::register(&utils.db, chat_id.0, msg.id.0, "INVOKE").await?;

    let mut query = Search::Default(text.into());
    if let Ok(command) = command {
        // safe to unwrap as /start and /help won't reach this statement
        query = Search::try_from(command).expect("cannot parse search type from command");
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
