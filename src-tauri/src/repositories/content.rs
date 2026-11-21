use rusqlite::{Connection, Result, params};
use serde::Serialize;

use super::watch::{self, Watch};

#[derive(Debug, Serialize)]
pub struct Category {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    pub id: i64,
    pub category_id: i64,
    pub title: String,
    pub cover_key: Option<String>,
    pub description: String,
    pub opinion: String,
    pub release_year: Option<i32>,
    pub watch: Watch,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeriesContents {
    pub items: Vec<Content>,
    pub categories: Vec<Category>,
    pub rating_average: Option<f64>,
    pub updated_at: String,
}

pub fn list(connection: &Connection, series_id: i64) -> Result<SeriesContents> {
    let mut query =
        connection.prepare("SELECT id, name FROM categories ORDER BY normalized_name, id")?;
    let categories = query
        .query_map([], |row| {
            Ok(Category {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?
        .collect::<Result<_>>()?;
    let mut query = connection.prepare(
        "SELECT c.id, c.category_id, c.title, c.cover_key, c.description, c.opinion, c.release_year,
         w.id, w.watched_episodes, w.status, w.started_on, w.finished_on, w.completed, w.active,
         c.total_episodes, c.rating_tenths,
         (SELECT COUNT(*) FROM watch_sessions previous WHERE previous.content_id = c.id AND previous.completed = 1)
         FROM contents c JOIN watch_sessions w ON w.content_id = c.id AND w.active = 1
         WHERE c.series_id = ?1 ORDER BY c.sort_order, c.id",
    )?;
    let items = query
        .query_map([series_id], |row| {
            Ok(Content {
                id: row.get(0)?,
                category_id: row.get(1)?,
                title: row.get(2)?,
                cover_key: row.get(3)?,
                description: row.get(4)?,
                opinion: row.get(5)?,
                release_year: row.get(6)?,
                watch: Watch {
                    session: watch::read_session(row, 7)?,
                    total_episodes: row.get(14)?,
                    rating_tenths: row.get(15)?,
                    completed_views: row.get(16)?,
                },
            })
        })?
        .collect::<Result<_>>()?;
    let (rating_average, updated_at) = connection.query_row(
        "SELECT (SELECT AVG(rating_tenths) / 10.0 FROM contents WHERE series_id = ?1),
         updated_at FROM series WHERE id = ?1",
        [series_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    Ok(SeriesContents {
        items,
        categories,
        rating_average,
        updated_at,
    })
}

pub fn category(connection: &Connection, name: &str, normalized: &str) -> Result<i64> {
    connection.execute(
        "INSERT INTO categories (name, normalized_name) VALUES (?1, ?2)
         ON CONFLICT(normalized_name) DO NOTHING",
        params![name, normalized],
    )?;
    connection.query_row(
        "SELECT id FROM categories WHERE normalized_name = ?1",
        [normalized],
        |row| row.get(0),
    )
}

pub struct ContentValues<'a> {
    pub category_id: i64,
    pub title: &'a str,
    pub cover_key: Option<&'a str>,
    pub description: &'a str,
    pub opinion: &'a str,
    pub release_year: Option<i32>,
}

pub fn save(
    connection: &Connection,
    series_id: i64,
    id: Option<i64>,
    values: ContentValues<'_>,
) -> Result<()> {
    let ContentValues {
        category_id,
        title,
        cover_key,
        description,
        opinion,
        release_year,
    } = values;
    if let Some(id) = id {
        connection.execute(
            "UPDATE contents SET category_id = ?1, title = ?2, cover_key = ?3, description = ?4,
             opinion = ?5, release_year = ?6, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
             WHERE series_id = ?7 AND id = ?8",
            params![
                category_id,
                title,
                cover_key,
                description,
                opinion,
                release_year,
                series_id,
                id
            ],
        )?;
    } else {
        connection.execute(
            "INSERT INTO contents (category_id, title, cover_key, description, opinion, release_year, series_id, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7,
             (SELECT COALESCE(MAX(sort_order) + 1, 0) FROM contents WHERE series_id = ?7))",
            params![category_id, title, cover_key, description, opinion, release_year, series_id],
        )?;
    }
    Ok(())
}

pub fn cover_key(connection: &Connection, series_id: i64, id: i64) -> Result<Option<String>> {
    connection.query_row(
        "SELECT cover_key FROM contents WHERE series_id = ?1 AND id = ?2",
        [series_id, id],
        |row| row.get(0),
    )
}

pub fn cover_keys(connection: &Connection, series_id: i64) -> Result<Vec<String>> {
    connection
        .prepare("SELECT cover_key FROM contents WHERE series_id = ?1 AND cover_key IS NOT NULL")?
        .query_map([series_id], |row| row.get(0))?
        .collect()
}

pub fn delete(connection: &Connection, series_id: i64, id: i64) -> Result<()> {
    connection.execute(
        "DELETE FROM contents WHERE series_id = ?1 AND id = ?2",
        [series_id, id],
    )?;
    Ok(())
}

pub fn order(connection: &Connection, series_id: i64) -> Result<Vec<i64>> {
    connection
        .prepare("SELECT id FROM contents WHERE series_id = ?1 ORDER BY sort_order, id")?
        .query_map([series_id], |row| row.get(0))?
        .collect()
}

pub fn write_order(connection: &Connection, series_id: i64, ids: &[i64]) -> Result<()> {
    let mut update = connection.prepare(
        "UPDATE contents SET sort_order = ?1,
        updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
        WHERE series_id = ?2 AND id = ?3 AND sort_order <> ?1",
    )?;
    for (position, id) in (0_i64..).zip(ids) {
        update.execute(params![position, series_id, id])?;
    }
    Ok(())
}

pub fn touch_series(connection: &Connection, series_id: i64) -> Result<()> {
    connection.execute(
        "UPDATE series SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = ?1",
        [series_id],
    )?;
    Ok(())
}
