use std::{
    error::Error,
    sync::Arc,
    ops::Index
};
use reqwest::Url;
use libgen::{
    Book,
    Utils,
    Search, get_ids
};
use teloxide::{
    prelude2::*,
    Bot,
    adaptors::AutoSend,
    types::{
        Message,
        InlineKeyboardButton,
        InlineKeyboardMarkup
    },
    utils::command::BotCommand 
};
use libgen::get_books;

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
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(id) = q.data {
        let book = get_ids(&utils.client, vec![id.parse().unwrap()]).await;
        let book = book.first().unwrap();

        let url = format!("http://gen.lib.rus.ec/book/index.php?md5={}", book.md5);
        let url_keyboard = make_url_keyboard(&url);
        
        match q.message {
            Some(Message { id, chat, .. }) => {
                log::info!("{} selected: {}", chat.id, book.md5);
                bot.edit_message_text(chat.id, id, book.pretty())
                    .reply_markup(url_keyboard).await?;
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
    utils: Arc<Utils>,
) ->Result<(), Box<dyn Error + Send + Sync>> {
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
        bot.edit_message_text(chat_id, msg.id, text).reply_markup(keyboard).await?;
    } else {
        bot.edit_message_text(chat_id, msg.id, "Sorry, I don't have any result for that...").await?;
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

fn make_url_keyboard(url: &str) -> InlineKeyboardMarkup {
    let url = Url::parse(url).unwrap();
    let button = InlineKeyboardButton::url("Download".to_string(), url);

    let keyboard = vec![vec![button]];    
    InlineKeyboardMarkup::new(keyboard)
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
                books.index((i.to_owned() - 1) as usize).id.to_owned())
            )
        }

        keyboard.push(row);
    }

    InlineKeyboardMarkup::new(keyboard)
}
