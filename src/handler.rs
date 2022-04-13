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
    utils::command::BotCommand 
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
    if let Some(download_data) = q.data {
        match q.message {
            Some(Message { id, chat, .. }) => {
                log::info!("{} requested link", chat.id);
                bot.edit_message_text(chat.id, id, download_data).await?;
            }
            None => {
                if let Some(id) = q.inline_message_id {
                    bot.edit_message_text_inline(id, download_data).await?;
                }
            }
        }
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

    log::info!("{} contacted bot: {}", chat_id, text);

    let msg = bot.send_message(chat_id, "🤖 Loading...").await?;

    let mut books: Option<Vec<Book>> = None;
    if let Ok(command) = Command::parse(text, "gactivitybot") {
        let q = match command {
            Command::Author(author) => {
                Some(Search::Author(author))
            }
            Command::Title(title) => {
                Some(Search::Title(title))
            }
            Command::ISBN(isbn) => {
                Some(Search::ISBN(isbn))
            }
            _ => { None }
        };

        if q.is_some() {
            books = Some(get_books(&utils.client, q.unwrap(), 5).await);
        } else {
            bot.send_message(m.chat.id, "Tell me what to look for! :)").await?;
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
            let keyboard = make_keyboard(&books);
            let text = make_message(&books);
            bot.edit_message_text(chat_id, msg.id, text).reply_markup(keyboard).await?;
        } else {
            bot.edit_message_text(chat_id, msg.id, "Sorry, I don't have any result for that...").await?;
        }
    }

    Ok(())
}

fn make_message(books: &Vec<Book>) -> String {
    let msg: String = books
        .iter()
        .enumerate()
        .map(|(i, b)| b.pretty_with_index(i+1) + "\n")
        .collect();
        
    msg
}

fn make_keyboard(books: &Vec<Book>) -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];
    let b_len: u8 = books.len() as u8;
    let range: Vec<_> = (1..b_len+1).collect();

    for indexes in range.chunks(5) {
        let mut row = Vec::new();
        for i in indexes {
            row.push(InlineKeyboardButton::callback(
                format!("{}", i),
                books[(i - 1) as usize].download_data()
            ))
        }

        keyboard.push(row);
    }

    InlineKeyboardMarkup::new(keyboard)
}