use std::path::Path;

use rusqlite::{Connection, Result as SqlResult};
use serde::Serialize;

use super::catalog::{CatalogError, open};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Statistics {
    series_count: i64,
    content_count: i64,
    favorites: i64,
    active_episodes: i64,
    past_episodes: i64,
    total_episodes: i64,
    completed_views: i64,
    average: Option<f64>,
    statuses: Vec<Count>,
    genres: Vec<TermCount>,
    categories: Vec<TermCount>,
    ranking: Vec<Rank>,
    ratings: Vec<Count>,
}

#[derive(Serialize)]
struct Count {
    label: String,
    count: i64,
}

#[derive(Serialize)]
struct TermCount {
    id: i64,
    label: String,
    count: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Rank {
    id: i64,
    title: String,
    average: f64,
    cover_key: Option<String>,
}

fn counts(connection: &Connection, sql: &str) -> SqlResult<Vec<Count>> {
    connection
        .prepare(sql)?
        .query_map([], |row| {
            Ok(Count {
                label: row.get(0)?,
                count: row.get(1)?,
            })
        })?
        .collect()
}

fn term_counts(connection: &Connection, sql: &str) -> SqlResult<Vec<TermCount>> {
    connection
        .prepare(sql)?
        .query_map([], |row| {
            Ok(TermCount {
                id: row.get(0)?,
                label: row.get(1)?,
                count: row.get(2)?,
            })
        })?
        .collect()
}

pub fn statistics(root: &Path) -> Result<Statistics, CatalogError> {
    let mut connection = open(root)?;
    let tx = connection.transaction()?;
    let (series_count, favorites) = tx.query_row(
        "SELECT COUNT(*), COALESCE(SUM(favorite), 0) FROM series",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let (content_count, average) = tx.query_row(
        "SELECT COUNT(*), AVG(rating_tenths) / 10.0 FROM contents",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let (active_episodes, past_episodes, completed_views) = tx.query_row(
        "SELECT COALESCE(SUM(watched_episodes) FILTER (WHERE active = 1), 0),
                COALESCE(SUM(watched_episodes) FILTER (WHERE active = 0), 0),
                COALESCE(SUM(completed), 0) FROM watch_sessions",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;
    let result = Statistics {
        series_count, content_count, favorites, active_episodes, past_episodes,
        total_episodes: active_episodes + past_episodes, completed_views, average,
        statuses: counts(&tx, "SELECT status, COUNT(*) FROM watch_sessions WHERE active = 1 GROUP BY status")?,
        genres: term_counts(&tx, "SELECT g.id, g.name, COUNT(sg.series_id) AS usage FROM genres g
            JOIN series_genres sg ON sg.genre_id = g.id GROUP BY g.id ORDER BY usage DESC, g.normalized_name, g.id")?,
        categories: term_counts(&tx, "SELECT a.id, a.name, COUNT(c.id) AS usage FROM categories a
            JOIN contents c ON c.category_id = a.id GROUP BY a.id ORDER BY usage DESC, a.normalized_name, a.id")?,
        ranking: tx.prepare("SELECT s.id, s.title, AVG(c.rating_tenths) / 10.0 AS score, s.cover_key
            FROM series s JOIN contents c ON c.series_id = s.id WHERE c.rating_tenths IS NOT NULL
            GROUP BY s.id ORDER BY score DESC, s.normalized_title, s.id LIMIT 10")?
            .query_map([], |row| Ok(Rank { id: row.get(0)?, title: row.get(1)?, average: row.get(2)?, cover_key: row.get(3)? }))?
            .collect::<SqlResult<_>>()?,
        ratings: counts(&tx, "SELECT CAST(rating_tenths / 10 AS TEXT), COUNT(*) FROM contents
            WHERE rating_tenths IS NOT NULL GROUP BY rating_tenths / 10 ORDER BY rating_tenths / 10")?,
    };
    tx.commit()?;
    Ok(result)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinueItem {
    id: i64,
    series_id: i64,
    series_title: String,
    title: String,
    category: String,
    cover_key: Option<String>,
    session_id: i64,
    watched_episodes: i64,
    total_episodes: Option<i64>,
    rating_tenths: Option<i64>,
}

pub fn continue_watching(root: &Path) -> Result<Vec<ContinueItem>, CatalogError> {
    let connection = open(root)?;
    Ok(connection
        .prepare(
            "SELECT c.id, s.id, s.title, c.title, a.name, c.cover_key, w.id,
                w.watched_episodes, c.total_episodes, c.rating_tenths
         FROM watch_sessions w JOIN contents c ON c.id = w.content_id
         JOIN series s ON s.id = c.series_id JOIN categories a ON a.id = c.category_id
         WHERE w.active = 1 AND w.status = 'watching'
         ORDER BY w.activity_at DESC, w.id DESC",
        )?
        .query_map([], |row| {
            Ok(ContinueItem {
                id: row.get(0)?,
                series_id: row.get(1)?,
                series_title: row.get(2)?,
                title: row.get(3)?,
                category: row.get(4)?,
                cover_key: row.get(5)?,
                session_id: row.get(6)?,
                watched_episodes: row.get(7)?,
                total_episodes: row.get(8)?,
                rating_tenths: row.get(9)?,
            })
        })?
        .collect::<SqlResult<_>>()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::watch;
    use serde_json::json;
    use tempfile::tempdir;

    #[test]
    fn aggregates_each_current_and_archived_session_once() {
        let temp = tempdir().unwrap();
        let root = temp.path();
        let empty = statistics(root).unwrap();
        assert_eq!(empty.content_count, 0);
        assert_eq!(empty.average, None);
        assert!(continue_watching(root).unwrap().is_empty());
        open(root).unwrap().execute_batch(
            "INSERT INTO series (title, normalized_title, favorite, cover_key)
             VALUES ('Órbita', 'orbita', 1, 'portada'), ('Sin notas', 'sin notas', 0, NULL);
             INSERT INTO categories (name, normalized_name) VALUES ('Corto', 'corto'), ('Temporada', 'temporada');
             INSERT INTO genres (name, normalized_name) VALUES ('Drama', 'drama'), ('Acción', 'accion');
             INSERT INTO series_genres VALUES (1, 1), (1, 2);
             INSERT INTO contents (series_id, category_id, title, sort_order, rating_tenths, total_episodes)
             VALUES (1, 1, 'Piloto', 0, 95, 2), (1, 2, 'Segundo', 1, NULL, NULL), (2, 1, 'Otro', 0, NULL, NULL);
             UPDATE watch_sessions SET watched_episodes = 1, status = 'watching', activity_at = '2020' WHERE content_id IN (1, 2);
             UPDATE watch_sessions SET activity_at = CASE content_id WHEN 1 THEN '2020' ELSE '2021' END WHERE active = 1;",
        ).unwrap();
        let queue = continue_watching(root).unwrap();
        assert_eq!(queue.iter().map(|item| item.id).collect::<Vec<_>>(), [2, 1]);
        let first = &queue[1];
        let step = |session_id, action| {
            watch::update(
                root,
                serde_json::from_value(json!({
                    "seriesId": 1, "contentId": 1, "sessionId": session_id, "action": action
                }))
                .unwrap(),
            )
            .unwrap()
        };
        step(first.session_id, json!({"kind": "step", "delta": 1}));
        assert_eq!(continue_watching(root).unwrap().len(), 1);
        let next = step(first.session_id, json!({"kind": "rewatch"}));
        assert_eq!(continue_watching(root).unwrap().len(), 1);
        step(
            next.items[0].watch.session.id,
            json!({"kind": "step", "delta": 1}),
        );
        let queue = continue_watching(root).unwrap();
        assert_eq!(queue[0].id, 1);
        assert_eq!(queue[0].watched_episodes, 1);
        assert_eq!(queue[1].total_episodes, None);
        open(root)
            .unwrap()
            .execute("UPDATE contents SET updated_at = '9999' WHERE id = 2", [])
            .unwrap();
        assert_eq!(continue_watching(root).unwrap()[0].id, 1);

        let stats = statistics(root).unwrap();
        assert_eq!(
            (stats.series_count, stats.content_count, stats.favorites),
            (2, 3, 1)
        );
        assert_eq!(
            (
                stats.active_episodes,
                stats.past_episodes,
                stats.total_episodes
            ),
            (2, 2, 4)
        );
        assert_eq!(stats.completed_views, 1);
        assert_eq!(stats.average, Some(9.5));
        assert_eq!(stats.ranking.len(), 1);
        assert_eq!(stats.ranking[0].id, 1);
        assert_eq!(stats.ranking[0].cover_key.as_deref(), Some("portada"));
        assert_eq!(
            (stats.ratings[0].label.as_str(), stats.ratings[0].count),
            ("9", 1)
        );
        assert_eq!(stats.categories[0].count, 2);
        assert_eq!(stats.categories[0].id, 1);
        assert_eq!(stats.genres.len(), 2);
        let catalog = crate::repositories::catalog(&open(root).unwrap()).unwrap();
        let pilot = catalog.series[0]
            .contents
            .iter()
            .find(|content| content.title == "Piloto")
            .unwrap();
        assert_eq!((pilot.category_id, pilot.rating_tenths), (1, Some(95)));
        assert_eq!(
            stats
                .statuses
                .iter()
                .find(|row| row.label == "watching")
                .unwrap()
                .count,
            2
        );
    }
}
