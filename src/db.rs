use rusqlite::{params, Connection, Result};

pub fn get_db(path: &str) -> Result<Connection> {
    let db = Connection::open(path)?;
    run_migrations(&db)?;

    Ok(db)
}

fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS analytics (
            user_id  INTEGER NOT NULL,
            msg_id   INTEGER NOT NULL,
            type     TEXT NOT NULL,
            utime    INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        );",
        [],
    )?;

    Ok(())
}

pub fn register(conn: &Connection, chat_id: i64, message_id: i32, atype: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO analytics (user_id, msg_id, type) VALUES (?,?,?)",
        params![chat_id, message_id, atype],
    )?;

    Ok(())
}
