use std::{error::Error, sync::Arc};
use libgen::{Book, Utils, Search};
use teloxide::{
    prelude2::*,
    Bot,
    adaptors::AutoSend,
    types::{
        Message,
        MessageKind, InlineKeyboardButton, InlineKeyboardMarkup
    },
    utils::command::BotCommand, payloads::SendMessageSetters
};

use libgen::get_books;

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    Start,
    ISBN(String),
    Title(String),
    Author(String)
}

pub async fn callback_handler(
    q: CallbackQuery,
    bot: AutoSend<Bot>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(version) = q.data {
        let text = format!("You chose: {}", version);

        match q.message {
            Some(Message { id, chat, .. }) => {
                bot.edit_message_text(chat.id, id, text).await?;
            }
            None => {
                if let Some(id) = q.inline_message_id {
                    bot.edit_message_text_inline(id, text).await?;
                }
            }
        }

        log::info!("You chose: {}", version);
    }

    Ok(())
}

pub async fn message_handler(
    bot: AutoSend<Bot>,
    m: Message,
    utils: Arc<Utils>,
) ->Result<(), Box<dyn Error + Send + Sync>> {
    let chat_id = m.chat_id();

    let text = match m.text() {
        Some(text) => text,
        None => { 
            return Ok(()); 
        }
    };

    let mut books: Option<Vec<Book>> = None;
    if let Ok(command) = Command::parse(text, "gactivitybot") {
        match command {
            Command::Start => {
                bot.send_message(m.chat.id, "Tell me what to look for! :)").await?;
            }
            Command::Author(author) => {
                books = Some(get_books(&utils.client, Search::Author(author), 5).await);
            }
            Command::Title(title) => {
                books = Some(get_books(&utils.client, Search::Title(title), 5).await);
            }
            Command::ISBN(isbn) => {
                books = Some(get_books(&utils.client, Search::ISBN(isbn), 5).await);
            }
        }
    } else {
        match m.kind {
            MessageKind::Common(_) => {
                books = Some(get_books(&utils.client, Search::Default(text.into()), 5).await);
            }
            _ => {}
        }
    }

    if let Some(books) = books {
        if books.len() > 0 {
            let keyboard = make_keyboard(books);
            bot.send_message(chat_id, "This is what I've found").reply_markup(keyboard).await?;
        } else {
            bot.send_message(chat_id, "Sorry, I don't have any result for that...").await?;
        }
    }

    Ok(())
}

fn make_keyboard(books: Vec<Book>) -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];

    for book in books {
        keyboard.push(vec![InlineKeyboardButton::callback(book.title.to_owned(), book.md5.to_owned())]);
    }

    InlineKeyboardMarkup::new(keyboard)
}