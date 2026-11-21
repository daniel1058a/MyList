use rusqlite::{Connection, Error, Result, Row, params, types::Type};
use serde::Serialize;

use crate::domain::watch::{Session, Status};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Watch {
    pub session: Session,
    pub total_episodes: Option<i64>,
    pub rating_tenths: Option<i64>,
    pub completed_views: i64,
}

pub fn read_session(row: &Row<'_>, offset: usize) -> Result<Session> {
    Ok(Session {
        id: row.get(offset)?,
        watched_episodes: row.get(offset + 1)?,
        status: read_status(row, offset + 2)?,
        started_on: row.get(offset + 3)?,
        finished_on: row.get(offset + 4)?,
        completed: row.get(offset + 5)?,
        active: row.get(offset + 6)?,
    })
}

pub(super) fn read_status(row: &Row<'_>, column: usize) -> Result<Status> {
    let status: String = row.get(column)?;
    Status::parse(&status).ok_or(Error::InvalidColumnType(
        column,
        "status".into(),
        Type::Text,
    ))
}

pub fn current(
    connection: &Connection,
    series_id: i64,
    content_id: i64,
) -> Result<(Session, Option<i64>, Option<i64>)> {
    connection.query_row(
        "SELECT w.id, w.watched_episodes, w.status, w.started_on, w.finished_on, w.completed, w.active,
         c.total_episodes, c.rating_tenths
         FROM contents c JOIN watch_sessions w ON w.content_id = c.id AND w.active = 1
         WHERE c.series_id = ?1 AND c.id = ?2",
        [series_id, content_id],
        |row| Ok((read_session(row, 0)?, row.get(7)?, row.get(8)?)),
    )
}

pub fn history(connection: &Connection, content_id: i64) -> Result<Vec<Session>> {
    connection
        .prepare(
            "SELECT id, watched_episodes, status, started_on, finished_on, completed, active
         FROM watch_sessions WHERE content_id = ?1 ORDER BY id DESC",
        )?
        .query_map([content_id], |row| read_session(row, 0))?
        .collect()
}

pub fn save_session(connection: &Connection, session: &Session) -> Result<()> {
    connection.execute(
        "UPDATE watch_sessions SET watched_episodes = ?1, status = ?2, started_on = ?3,
         finished_on = ?4, completed = ?5 WHERE id = ?6",
        params![
            session.watched_episodes,
            session.status.as_str(),
            session.started_on,
            session.finished_on,
            session.completed,
            session.id
        ],
    )?;
    Ok(())
}

pub fn start_next(connection: &Connection, content_id: i64, previous_id: i64) -> Result<()> {
    connection.execute(
        "UPDATE watch_sessions SET active = 0 WHERE id = ?1",
        [previous_id],
    )?;
    connection.execute(
        "INSERT INTO watch_sessions (content_id) VALUES (?1)",
        [content_id],
    )?;
    Ok(())
}

pub fn save_content_watch(
    connection: &Connection,
    id: i64,
    total: Option<i64>,
    rating: Option<i64>,
) -> Result<()> {
    connection.execute(
        "UPDATE contents SET total_episodes = ?1, rating_tenths = ?2,
         updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?3",
        params![total, rating, id],
    )?;
    Ok(())
}
