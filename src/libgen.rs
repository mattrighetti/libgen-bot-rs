pub mod types;

use std::sync::Mutex;

use reqwest::Client;
use rusqlite::{Connection, params, Result};
use select::document::Document;
use select::predicate::Attr;

use crate::db::get_db;

use types::*;

const LIBGEN_URL: &str = "https://libgen.is/search.php";
const LIBGEN_API_URL: &str = "https://libgen.is/json.php";

pub struct Utils {
    pub client: Client,
    pub db: Mutex<Connection>,
}

impl Utils {
    pub fn new(db_path: String) -> Self {
        let conn = get_db(Some(db_path.as_str())).unwrap();

        Utils {
            client: Client::new(),
            db: Mutex::new(conn),
        }
    }

    pub fn register(&self, chat_id: i64, message_id: i32, atype: &str) -> Result<()> {
        let lock = self.db.lock().unwrap();
        lock.execute("INSERT INTO analytics (user_id, msg_id, type) VALUES (?,?,?)", params![chat_id, message_id, atype])?;

        Ok(())
    }
}

pub async fn search(client: &Client, query: Search, limit: usize) -> Vec<u32> {
    let res = client.get(LIBGEN_URL).query(&query.search_params()).send().await.unwrap();
    let text = res.text().await.unwrap();
    let cursor = std::io::Cursor::new(text.as_str());
    let doc = Document::from_read(cursor).unwrap();
    
    let mut ids = Vec::new();
    for node in doc.find(Attr("valign", "top")).skip(1).take(limit) {
        let id = node.descendants().take(1).next().unwrap().text();
        
        match id.parse::<u32>() {
            Ok(id) => { ids.push(id) }
            Err(_) => {}
        }
    }

    ids
}

pub async fn get_ids(client: &Client, ids: Vec<u32>) -> Vec<Book> {
    assert!(ids.len() > 0);
    let ids = ids.iter()
        .map(|z| z.to_string())
        .collect::<Vec<String>>()
        .join(",");

    let params: Vec<(String, String)> = vec![
        ("fields".into(), "id,title,author,year,extension,md5".into()),
        ("ids".into(), ids)
    ];

    let res = client
        .get(LIBGEN_API_URL)
        .query(&params)
        .send()
        .await
        .unwrap();
    
    res.json::<Vec<Book>>().await.unwrap()
}

pub async fn get_books(client: &Client, query: Search, limit: usize) -> Vec<Book> {
    let ids = search(client, query, limit).await;
    if ids.len() > 0 {
        return get_ids(client, ids).await
    }

    vec![]
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_search_invalid_def() {
        let client = reqwest::Client::new();
        let query = Search::Default("Ahaha".into());

        let ids = search(&client, query, 5).await;

        assert_eq!(ids.len(), 0);
    }

    #[tokio::test]
    async fn test_search_title() {
        let client = reqwest::Client::new();
        let query = Search::Title("Rust Programming".into());

        let ids = search(&client, query, 25).await;
        let expected: Vec<u32> = vec![1486260, 1527378, 1729710, 2158512, 2167798];

        assert_eq!(ids.len(), 25);

        let mut trunc_ids = ids.iter().take(5);
        assert_eq!(trunc_ids.next(), Some(&expected[0]));
        assert_eq!(trunc_ids.next(), Some(&expected[1]));
        assert_eq!(trunc_ids.next(), Some(&expected[2]));
        assert_eq!(trunc_ids.next(), Some(&expected[3]));
        assert_eq!(trunc_ids.next(), Some(&expected[4]));
    }

    #[tokio::test]
    async fn test_search_author() {
        let client = reqwest::Client::new();
        let query = Search::Author("Orendorff".into());

        let ids = search(&client, query, 5).await;
        let expected: Vec<u32> = vec![1486260, 1527378, 2158512, 2167798, 2917089];

        assert_eq!(ids, expected);
    }

    #[tokio::test]
    async fn test_search_default() {
        let client = reqwest::Client::new();
        let query = Search::Default("Rust Programming".into());

        let ids = search(&client, query, 5).await;
        let expected: Vec<u32> = vec![349771, 1486260, 1527378, 1729710, 1980775];

        assert_eq!(ids, expected);
    }

    #[tokio::test]
    async fn test_search_isbn() {
        let client = reqwest::Client::new();
        let query = Search::ISBN("978-0132350884".into());

        let ids = search(&client, query, 5).await;
        let expected: Vec<u32> = vec![207243, 1489412, 1525091, 2228027, 2324753];

        assert_eq!(ids, expected);
    }

    #[tokio::test]
    async fn test_search_empty_isbn() {
        let client = reqwest::Client::new();
        let query = Search::ISBN("1823499234".into());

        let ids = search(&client, query, 5).await;
        let expected: Vec<u32> = vec![];

        assert_eq!(ids, expected);
    }

    #[tokio::test]
    async fn test_get_ids() {
        let client = reqwest::Client::new();
        let result = get_ids(&client, vec![349771, 1486260, 1527378, 1729710, 1980775]).await;
        
        assert_eq!(result.len(), 5);
    }

    #[tokio::test]
    async fn test_get_ids_max() {
        let client = reqwest::Client::new();
        let result = get_ids(&client, vec![349771, 1486260, 1527378, 1729710, 1980775, 349771, 349771, 349771]).await;
        
        assert_eq!(result.len(), 5);
    }

    #[tokio::test]
    async fn test_invalid_get_books_def() {
        let client = reqwest::Client::new();
        let query = Search::Default("Ahaha".into());

        let books = get_books(&client, query, 5).await;

        assert_eq!(books.len(), 0);
    }
}