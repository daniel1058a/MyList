use std::collections::BTreeMap;

use crate::domain::watch::Status;
use rusqlite::{Connection, Result, params};
use serde::Serialize;

pub mod content;
pub mod taxonomy;
pub mod watch;

#[derive(Debug, Serialize)]
pub struct Genre {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Series {
    pub id: i64,
    pub title: String,
    pub cover_key: Option<String>,
    pub description: String,
    pub panorama_key: Option<String>,
    pub favorite: bool,
    pub genre_ids: Vec<i64>,
    pub rating_average: Option<f64>,
    pub created_at: String,
    pub updated_at: String,
    pub contents: Vec<CatalogContent>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogContent {
    pub title: String,
    pub status: Status,
    pub category_id: i64,
    pub rating_tenths: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct Catalog {
    pub series: Vec<Series>,
    pub genres: Vec<Genre>,
}

pub fn catalog(connection: &Connection) -> Result<Catalog> {
    let mut query =
        connection.prepare("SELECT id, name FROM genres ORDER BY normalized_name, id")?;
    let genres = query
        .query_map([], |row| {
            Ok(Genre {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?
        .collect::<Result<_>>()?;
    let mut query = connection.prepare(
        "SELECT s.id, s.title, s.cover_key, s.favorite,
         (SELECT AVG(c.rating_tenths) / 10.0 FROM contents c WHERE c.series_id = s.id),
         s.created_at, s.updated_at, s.description, s.panorama_key
         FROM series s ORDER BY s.normalized_title, s.id",
    )?;
    let mut series: Vec<Series> = query
        .query_map([], |row| {
            Ok(Series {
                id: row.get(0)?,
                title: row.get(1)?,
                cover_key: row.get(2)?,
                favorite: row.get(3)?,
                genre_ids: Vec::new(),
                rating_average: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
                description: row.get(7)?,
                panorama_key: row.get(8)?,
                contents: Vec::new(),
            })
        })?
        .collect::<Result<_>>()?;
    let by_id: BTreeMap<_, _> = series
        .iter()
        .enumerate()
        .map(|(index, item)| (item.id, index))
        .collect();
    let mut links = connection
        .prepare("SELECT series_id, genre_id FROM series_genres ORDER BY series_id, genre_id")?;
    let mut rows = links.query([])?;
    while let Some(row) = rows.next()? {
        let id: i64 = row.get(0)?;
        if let Some(&index) = by_id.get(&id) {
            series[index].genre_ids.push(row.get(1)?);
        }
    }
    let mut content_titles = connection.prepare(
        "SELECT c.series_id, c.title, w.status, c.category_id, c.rating_tenths FROM contents c
         JOIN watch_sessions w ON w.content_id = c.id AND w.active = 1",
    )?;
    let mut rows = content_titles.query([])?;
    while let Some(row) = rows.next()? {
        let id: i64 = row.get(0)?;
        if let Some(&index) = by_id.get(&id) {
            series[index].contents.push(CatalogContent {
                title: row.get(1)?,
                status: watch::read_status(row, 2)?,
                category_id: row.get(3)?,
                rating_tenths: row.get(4)?,
            });
        }
    }
    Ok(Catalog { series, genres })
}

pub struct SeriesKeys {
    pub normalized_title: String,
    pub cover: Option<String>,
    pub panorama: Option<String>,
}

pub fn series_keys(connection: &Connection, id: i64) -> Result<SeriesKeys> {
    connection.query_row(
        "SELECT normalized_title, cover_key, panorama_key FROM series WHERE id = ?1",
        [id],
        |row| {
            Ok(SeriesKeys {
                normalized_title: row.get(0)?,
                cover: row.get(1)?,
                panorama: row.get(2)?,
            })
        },
    )
}

pub fn duplicate(connection: &Connection, title: &str, own_id: Option<i64>) -> Result<bool> {
    connection.query_row(
        "SELECT EXISTS (SELECT 1 FROM series WHERE normalized_title = ?1 AND id IS NOT ?2)",
        params![title, own_id],
        |row| row.get(0),
    )
}

pub struct SeriesValues<'a> {
    pub title: &'a str,
    pub normalized: &'a str,
    pub description: &'a str,
    pub cover: Option<&'a str>,
    pub panorama: Option<&'a str>,
}

pub fn save_series(
    connection: &Connection,
    id: Option<i64>,
    values: SeriesValues<'_>,
) -> Result<i64> {
    let fields = params![
        values.title,
        values.normalized,
        values.cover,
        values.description,
        values.panorama
    ];
    if let Some(id) = id {
        connection.execute(
            "UPDATE series SET title = ?1, normalized_title = ?2, cover_key = ?3,
             description = ?4, panorama_key = ?5,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = ?6",
            params![
                values.title,
                values.normalized,
                values.cover,
                values.description,
                values.panorama,
                id
            ],
        )?;
        Ok(id)
    } else {
        connection.query_row(
            "INSERT INTO series (title, normalized_title, cover_key, description, panorama_key)
             VALUES (?1, ?2, ?3, ?4, ?5) RETURNING id",
            fields,
            |row| row.get(0),
        )
    }
}

pub fn set_genres(
    connection: &Connection,
    id: i64,
    genres: &BTreeMap<String, String>,
) -> Result<()> {
    connection.execute("DELETE FROM series_genres WHERE series_id = ?1", [id])?;
    for (normalized, name) in genres {
        connection.execute(
            "INSERT INTO genres (name, normalized_name) VALUES (?1, ?2) ON CONFLICT(normalized_name) DO NOTHING",
            params![name, normalized],
        )?;
        connection.execute(
            "INSERT INTO series_genres (series_id, genre_id) SELECT ?1, id FROM genres WHERE normalized_name = ?2",
            params![id, normalized],
        )?;
    }
    Ok(())
}

pub fn set_favorite(connection: &Connection, id: i64, favorite: bool) -> Result<usize> {
    connection.execute(
        "UPDATE series SET favorite = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = ?2",
        params![favorite, id],
    )
}

pub fn delete_series(connection: &Connection, id: i64) -> Result<()> {
    connection.execute("DELETE FROM series WHERE id = ?1", [id])?;
    Ok(())
}
