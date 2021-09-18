use rusqlite::{params, Connection, Result};

#[derive(Clone, Debug)]
pub struct NotVerifiedAndMessages {
    pub is_not_verified: u32,
    pub messages: u32,
}

#[derive(Clone, Debug)]
pub struct Messages {
    pub mess_id: i32,
    pub user_id: i64,
    pub forward_mess_id: i32,
}

pub(crate) fn db() -> Connection {
    Connection::open("store.db").expect("Database connection error")
}

pub(crate) fn init() -> Result<()> {
    let conn = db();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS members (
             user_id            INTEGER PRIMARY KEY,
             verified           BOOLEAN NOT NULL
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS messages (
             mess_id            INTEGER PRIMARY KEY,
             user_id            INTEGER NOT NULL,
             forward_mess_id    INTEGER NOT NULL
        )",
        [],
    )?;
    Ok(())
}

pub(crate) fn add_member(user_id: i64) -> Result<()> {
    let conn = db();
    conn.execute(
        "INSERT OR IGNORE INTO members (user_id, verified)
        values (?1, ?2)",
        params![user_id, false],
    )?;
    Ok(())
}

pub(crate) fn add_message(mess_id: i32, user_id: i64) -> Result<()> {
    let conn = db();
    conn.execute(
        "INSERT OR IGNORE INTO messages (mess_id, user_id, forward_mess_id)
        values (?1, ?2, ?3)",
        params![mess_id, user_id, 0],
    )?;
    Ok(())
}

pub(crate) fn get_member_status_and_messages(
    user_id: i64,
) -> Result<Option<NotVerifiedAndMessages>> {
    let conn = db();
    let mut stmt =
        conn.prepare("SELECT * FROM (SELECT COUNT(*) FROM members WHERE user_id = ?1 AND verified = 0), (SELECT COUNT(*) FROM messages WHERE user_id = ?1)")?;
    return stmt
        .query_map(params![user_id], |row| {
            Ok(NotVerifiedAndMessages {
                is_not_verified: row.get(0)?,
                messages: row.get(1)?,
            })
        })?
        .next()
        .transpose();
}

pub(crate) fn set_member_verified(user_id: i64) -> Result<()> {
    let conn = db();
    conn.execute(
        "UPDATE members SET verified = 1 WHERE user_id = ?1;\
        DELETE FROM messages WHERE user_id = ?1",
        params![user_id],
    )?;
    Ok(())
}

pub(crate) fn get_member_messages(user_id: i64) -> Result<Vec<Messages>> {
    let conn = db();
    let mut stmt = conn.prepare("SELECT * FROM messages WHERE user_id = ?1")?;
    let rows = stmt.query_map(params![user_id], |row| {
        Ok(Messages {
            mess_id: row.get(0)?,
            user_id: row.get(1)?,
            forward_mess_id: row.get(2)?,
        })
    })?;
    let mut messages = Vec::new();
    for mess in rows {
        messages.push(mess?);
    }
    Ok(messages)
}

pub(crate) fn set_message_forward(mess_id: i32, forward_id: i32) -> Result<()> {
    log::error!(
        "set_message_forward, mess_id: {}, forward_id: {}",
        mess_id,
        forward_id
    );
    let conn = db();
    conn.execute(
        "UPDATE messages SET forward_mess_id = ?1 WHERE mess_id = ?2;",
        params![forward_id, mess_id],
    )?;
    Ok(())
}
