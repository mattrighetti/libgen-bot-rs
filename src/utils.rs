use std::ops::Index;
use reqwest::Url;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};
use crate::libgen::types::Book;

pub fn make_message(books: &[Book]) -> String {
    let msg: String = books
        .iter()
        .enumerate()
        .map(|(i, b)| b.pretty_with_index(i+1) + "\n")
        .collect();
        
    msg
}

pub fn make_url_keyboard(url: &str) -> InlineKeyboardMarkup {
    let url = Url::parse(url).unwrap();
    let button = InlineKeyboardButton::url("Download".to_string(), url);

    let keyboard = vec![vec![button]];    
    InlineKeyboardMarkup::new(keyboard)
}

pub fn make_keyboard(books: &Vec<Book>) -> InlineKeyboardMarkup {
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
