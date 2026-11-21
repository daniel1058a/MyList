use std::{
    collections::{BTreeMap, HashSet},
    path::Path,
};

use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};

use crate::domain::watch::Status;

use super::catalog::{CatalogError, open};

const KEY: &str = "catalog.preferences.v1";

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum GenreMode {
    #[default]
    All,
    Any,
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum SortBy {
    #[default]
    Title,
    Rating,
    Created,
    Updated,
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Direction {
    #[default]
    Asc,
    Desc,
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
struct ContentFilters {
    category_id: Option<i64>,
    status: Option<Status>,
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Theme {
    #[default]
    Dark,
    Light,
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum CoverSize {
    Small,
    #[default]
    Medium,
    Large,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    #[default]
    Es,
    En,
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct Preferences {
    query: String,
    genre_ids: Vec<i64>,
    genre_mode: GenreMode,
    status: Option<Status>,
    favorite_only: bool,
    sort_by: SortBy,
    direction: Direction,
    content_filters: BTreeMap<i64, ContentFilters>,
    theme: Theme,
    cover_size: CoverSize,
    pub language: Language,
}

fn validate(preferences: &Preferences) -> Result<(), CatalogError> {
    if preferences.query.chars().count() > 200
        || preferences.query.chars().any(char::is_control)
        || preferences.genre_ids.iter().any(|id| *id <= 0)
        || preferences.content_filters.iter().any(|(id, filters)| {
            *id <= 0 || filters.category_id.is_some_and(|category| category <= 0)
        })
    {
        return Err(CatalogError::InvalidPreferences);
    }
    Ok(())
}

fn ids(connection: &Connection, sql: &str) -> rusqlite::Result<HashSet<i64>> {
    connection
        .prepare(sql)?
        .query_map([], |row| row.get(0))?
        .collect()
}

fn retain_existing_filters(
    connection: &Connection,
    preferences: &mut Preferences,
) -> rusqlite::Result<()> {
    if !preferences.genre_ids.is_empty() {
        let genres = ids(connection, "SELECT id FROM genres")?;
        preferences.genre_ids.retain(|id| genres.contains(id));
    }
    if !preferences.content_filters.is_empty() {
        let series = ids(connection, "SELECT id FROM series")?;
        let categories = ids(connection, "SELECT id FROM categories")?;
        preferences.content_filters.retain(|id, filters| {
            if filters
                .category_id
                .is_some_and(|id| !categories.contains(&id))
            {
                filters.category_id = None;
            }
            series.contains(id) && (filters.category_id.is_some() || filters.status.is_some())
        });
    }
    Ok(())
}

pub fn access(
    data_dir: &Path,
    preferences: Option<Preferences>,
) -> Result<Preferences, CatalogError> {
    let mut connection = open(data_dir)?;
    if let Some(mut preferences) = preferences {
        validate(&preferences)?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        preferences.genre_ids.sort_unstable();
        preferences.genre_ids.dedup();
        retain_existing_filters(&transaction, &mut preferences)?;
        let value =
            serde_json::to_string(&preferences).map_err(|_| CatalogError::InvalidPreferences)?;
        transaction.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![KEY, value],
        )?;
        transaction.commit()?;
        return Ok(preferences);
    }
    let transaction = connection.transaction()?;
    let value: Option<String> = transaction
        .query_row("SELECT value FROM settings WHERE key = ?1", [KEY], |row| {
            row.get(0)
        })
        .optional()?;
    let Some(value) = value else {
        return Ok(Preferences::default());
    };
    let mut preferences =
        serde_json::from_str(&value).map_err(|_| CatalogError::InvalidPreferences)?;
    validate(&preferences)?;
    retain_existing_filters(&transaction, &mut preferences)?;
    transaction.commit()?;
    Ok(preferences)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn persists_validated_preferences_and_preserves_other_settings() {
        let directory = tempdir().unwrap();
        let root = directory.path();
        assert_eq!(access(root, None).unwrap(), Preferences::default());
        open(root)
            .unwrap()
            .execute_batch(
                "INSERT INTO settings VALUES ('other', 'true');
                 INSERT INTO genres (id, name, normalized_name) VALUES (1, 'Uno', 'uno'), (3, 'Tres', 'tres');
                 INSERT INTO categories (id, name, normalized_name) VALUES (4, 'Cuatro', 'cuatro');
                 INSERT INTO series (id, title, normalized_title) VALUES (2, 'Serie', 'serie');",
            )
            .unwrap();
        let value = serde_json::from_str(
            r#"{
            "query":"Película","genreIds":[3,1,3],"genreMode":"any","status":"watching",
            "favoriteOnly":true,"sortBy":"rating","direction":"desc",
            "contentFilters":{"2":{"categoryId":4,"status":"finished"},"3":{}},
            "theme":"light","coverSize":"large"
        }"#,
        )
        .unwrap();
        let saved = access(root, Some(value)).unwrap();
        assert_eq!(saved.genre_ids, [1, 3]);
        assert_eq!(saved.content_filters.len(), 1);
        assert_eq!(saved.theme, Theme::Light);
        assert_eq!(saved.cover_size, CoverSize::Large);
        assert_eq!(access(root, None).unwrap(), saved);
        let invalid = Preferences {
            genre_ids: vec![0],
            ..Preferences::default()
        };
        assert_eq!(
            access(root, Some(invalid)).unwrap_err(),
            CatalogError::InvalidPreferences
        );
        assert_eq!(access(root, None).unwrap(), saved);
        let connection = open(root).unwrap();
        assert_eq!(
            connection
                .query_row(
                    "SELECT value FROM settings WHERE key = 'other'",
                    [],
                    |row| row.get::<_, String>(0)
                )
                .unwrap(),
            "true"
        );
        connection
            .execute_batch("DELETE FROM genres WHERE id=1; DELETE FROM categories WHERE id=4;")
            .unwrap();
        let restored = access(root, None).unwrap();
        assert_eq!(restored.genre_ids, [3]);
        assert_eq!(restored.content_filters[&2].category_id, None);
        assert_eq!(restored.content_filters[&2].status, Some(Status::Finished));
        assert_eq!(restored.theme, Theme::Light);
    }

    #[test]
    fn projects_content_titles_and_only_active_session_states() {
        let directory = tempdir().unwrap();
        let connection = open(directory.path()).unwrap();
        connection
            .execute_batch(
                "INSERT INTO series(title, normalized_title) VALUES ('Serie', 'serie');
             INSERT INTO categories(name, normalized_name) VALUES ('Película', 'pelicula');
             INSERT INTO contents(series_id,category_id,title,sort_order) VALUES (1,1,'Órbita',0);
             UPDATE watch_sessions SET status='finished', completed=1, active=0;
             INSERT INTO watch_sessions(content_id,status) VALUES (1,'watching');",
            )
            .unwrap();
        let catalog = crate::repositories::catalog(&connection).unwrap();
        assert_eq!(catalog.series[0].contents.len(), 1);
        assert_eq!(catalog.series[0].contents[0].title, "Órbita");
        assert_eq!(catalog.series[0].contents[0].status, Status::Watching);
    }
}
