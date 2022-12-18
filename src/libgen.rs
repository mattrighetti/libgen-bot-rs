pub mod types;

use std::error::Error;
use std::sync::Mutex;

use reqwest::Client;
use rusqlite::{Connection, params};
use select::document::Document;
use select::predicate::Attr;

use crate::db::get_db;

use types::*;

const LIBGEN_URL: &str = "https://libgen.is/search.php";
const LIBGEN_API_URL: &str = "https://libgen.is/json.php";

type Result<T> = ::std::result::Result<T, Box<dyn Error + Send + Sync>>;

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

    pub fn register(&self, chat_id: i64, message_id: i32, atype: &str) -> rusqlite::Result<()> {
        let lock = self.db.lock().unwrap();
        lock.execute("INSERT INTO analytics (user_id, msg_id, type) VALUES (?,?,?)", params![chat_id, message_id, atype])?;

        Ok(())
    }
}

pub async fn search(client: &Client, query: Search, limit: usize) -> Result<Vec<u32>> {
    let res = client.get(LIBGEN_URL)
        .query(&query.search_params())
        .send()
        .await?;

    let html = res.text().await?;
    let doc = Document::from(html.as_str());

    let ids = doc.find(Attr("valign", "top"))
        .skip(1)
        .take(limit)
        .filter_map(|n| {
            let first_descendant = n.descendants().take(1).next();
            if let Some(fd) = first_descendant {
                if let Ok(val) = fd.text().parse::<u32>() {
                    return Some(val)
                }
            }

            None
        })
        .collect();

    Ok(ids)
}

pub async fn get_ids(client: &Client, ids: Vec<u32>) -> Result<Vec<Book>> {
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
        .await?;

    let books = res.json::<Vec<Book>>().await?;

    Ok(books)
}

pub async fn get_books(client: &Client, query: Search, limit: usize) -> Result<Vec<Book>> {
    let ids = search(client, query, limit).await?;
    if ids.len() > 0 {
       return get_ids(client, ids).await
    }

    Ok(vec![])
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_search_invalid_def() {
        let client = reqwest::Client::new();
        let query = Search::Default("Ahaha".into());

        let ids = search(&client, query, 5).await.unwrap();

        assert_eq!(ids.len(), 0);
    }

    #[tokio::test]
    async fn test_search_title() {
        let client = reqwest::Client::new();
        let query = Search::Title("Rust Programming".into());

        let ids = search(&client, query, 25).await.unwrap();
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

        let ids = search(&client, query, 5).await.unwrap();
        let expected: Vec<u32> = vec![1486260, 1527378, 2158512, 2167798, 2917089];

        assert_eq!(ids, expected);
    }

    #[tokio::test]
    async fn test_search_default() {
        let client = reqwest::Client::new();
        let query = Search::Default("Rust Programming".into());

        let ids = search(&client, query, 5).await.unwrap();
        let expected: Vec<u32> = vec![349771, 1486260, 1527378, 1729710, 1980775];

        assert_eq!(ids, expected);
    }

    #[tokio::test]
    async fn test_search_isbn() {
        let client = reqwest::Client::new();
        let query = Search::ISBN("978-0132350884".into());

        let ids = search(&client, query, 5).await.unwrap();
        let expected: Vec<u32> = vec![207243, 1489412, 1525091, 2228027, 2324753];

        assert_eq!(ids, expected);
    }

    #[tokio::test]
    async fn test_search_empty_isbn() {
        let client = reqwest::Client::new();
        let query = Search::ISBN("1823499234".into());

        let ids = search(&client, query, 5).await.unwrap();
        let expected: Vec<u32> = vec![];

        assert_eq!(ids, expected);
    }

    #[tokio::test]
    async fn test_get_ids() {
        let client = reqwest::Client::new();
        let result = get_ids(&client, vec![349771, 1486260, 1527378, 1729710, 1980775]).await.unwrap();

        assert_eq!(result.len(), 5);
    }

    #[tokio::test]
    async fn test_get_ids_max() {
        let client = reqwest::Client::new();
        let result = get_ids(&client, vec![349771, 1486260, 1527378, 1729710, 1980775, 349771, 349771, 349771]).await.unwrap();

        assert_eq!(result.len(), 5);
    }

    #[tokio::test]
    async fn test_invalid_get_books_def() {
        let client = reqwest::Client::new();
        let query = Search::Default("Ahaha".into());

        let books = get_books(&client, query, 5).await.unwrap();

        assert_eq!(books.len(), 0);
    }
}
