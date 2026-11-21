use std::{fs, path::Path, time::Duration};

use rusqlite::{Connection, TransactionBehavior};

pub(crate) const DATABASE_NAME: &str = "database.sqlite3";
const MIGRATIONS: &[(i32, &str)] = &[
    (1, include_str!("../../migrations/0001_settings.sql")),
    (2, include_str!("../../migrations/0002_catalog.sql")),
    (3, include_str!("../../migrations/0003_contents.sql")),
    (4, include_str!("../../migrations/0004_watch.sql")),
    (5, include_str!("../../migrations/0005_series_detail.sql")),
    (6, include_str!("../../migrations/0006_watch_activity.sql")),
];

#[derive(Debug, PartialEq, Eq)]
pub enum DatabaseError {
    CreateDirectory,
    Open,
    Configure,
    ReadSchema,
    UnsupportedSchema,
    Migrate,
}

pub fn open(data_dir: &Path) -> Result<Connection, DatabaseError> {
    fs::create_dir_all(data_dir).map_err(|_| DatabaseError::CreateDirectory)?;
    let mut connection =
        Connection::open(data_dir.join(DATABASE_NAME)).map_err(|_| DatabaseError::Open)?;
    connection
        .busy_timeout(Duration::from_secs(5))
        .map_err(|_| DatabaseError::Configure)?;
    connection
        .pragma_update(None, "foreign_keys", true)
        .map_err(|_| DatabaseError::Configure)?;
    migrate(&mut connection)?;
    Ok(connection)
}

pub fn schema_version(connection: &Connection) -> Result<i32, DatabaseError> {
    connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|_| DatabaseError::ReadSchema)
}

fn migrate(connection: &mut Connection) -> Result<(), DatabaseError> {
    let latest = latest_schema();
    let current = schema_version(connection)?;
    if !(0..=latest).contains(&current) {
        return Err(DatabaseError::UnsupportedSchema);
    }
    if current == latest {
        return Ok(());
    }

    migrate_to(connection, latest)
}

pub(crate) fn latest_schema() -> i32 {
    MIGRATIONS
        .last()
        .expect("Se requiere una migración inicial")
        .0
}

pub(crate) fn migrate_to(connection: &mut Connection, latest: i32) -> Result<(), DatabaseError> {
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|_| DatabaseError::Migrate)?;
    // Otra instancia puede haber migrado la base antes de adquirir el bloqueo.
    let current = schema_version(&transaction)?;
    if !(0..=latest).contains(&current) {
        return Err(DatabaseError::UnsupportedSchema);
    }
    for &(version, sql) in MIGRATIONS
        .iter()
        .filter(|(version, _)| *version > current && *version <= latest)
    {
        transaction
            .execute_batch(sql)
            .map_err(|_| DatabaseError::Migrate)?;
        transaction
            .pragma_update(None, "user_version", version)
            .map_err(|_| DatabaseError::Migrate)?;
    }
    transaction.commit().map_err(|_| DatabaseError::Migrate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn upgrades_version_five_with_legacy_activity_and_preserved_watch_state() {
        let temp = tempdir().unwrap();
        let connection = Connection::open(temp.path().join(DATABASE_NAME)).unwrap();
        for (_, sql) in &MIGRATIONS[..5] {
            connection.execute_batch(sql).unwrap();
        }
        connection.execute_batch(
            "PRAGMA user_version = 5;
             INSERT INTO series (title, normalized_title, description) VALUES ('Serie', 'serie', 'Descripción');
             INSERT INTO categories (name, normalized_name) VALUES ('Corto', 'corto');
             INSERT INTO contents (series_id, category_id, title, sort_order, updated_at)
             VALUES (1, 1, 'Parte', 0, '2025-01-01T00:00:00.000Z');
             UPDATE watch_sessions SET watched_episodes = 2, status = 'watching' WHERE content_id = 1;",
        ).unwrap();
        drop(connection);
        let connection = open(temp.path()).unwrap();
        let activity: (String, i64, String) = connection
            .query_row(
                "SELECT activity_at, watched_episodes, status FROM watch_sessions WHERE active = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(
            activity,
            ("2025-01-01T00:00:00.000Z".into(), 2, "watching".into())
        );
        assert_eq!(schema_version(&connection).unwrap(), 6);
        assert_eq!(
            crate::repositories::catalog(&connection).unwrap().series[0].description,
            "Descripción"
        );
    }

    #[test]
    fn upgrades_version_four_preserving_series_relationships_and_preferences() {
        let temp = tempdir().unwrap();
        let connection = Connection::open(temp.path().join(DATABASE_NAME)).unwrap();
        for (_, sql) in &MIGRATIONS[..4] {
            connection.execute_batch(sql).unwrap();
        }
        connection.execute_batch(
            r#"PRAGMA user_version = 4;
             INSERT INTO series (title, normalized_title, cover_key, favorite) VALUES ('Serie', 'serie', 'original', 1);
             INSERT INTO genres (name, normalized_name) VALUES ('Drama', 'drama');
             INSERT INTO series_genres VALUES (1, 1);
             INSERT INTO categories (name, normalized_name) VALUES ('Película', 'pelicula');
             INSERT INTO contents (series_id, category_id, title, rating_tenths, sort_order)
             VALUES (1, 1, 'Parte', 90, 2);
             UPDATE watch_sessions SET watched_episodes = 3, status = 'watching' WHERE content_id = 1;
             INSERT INTO settings VALUES ('catalog.preferences.v1',
                '{"theme":"light","coverSize":"large","genreIds":[1],"contentFilters":{"1":{"categoryId":1,"status":"watching"}}}');"#,
        ).unwrap();
        drop(connection);

        let connection = open(temp.path()).unwrap();
        assert_eq!(schema_version(&connection).unwrap(), 6);
        let catalog = crate::repositories::catalog(&connection).unwrap();
        let series = &catalog.series[0];
        assert_eq!(series.cover_key.as_deref(), Some("original"));
        assert!(series.description.is_empty() && series.panorama_key.is_none());
        assert!(series.favorite);
        assert_eq!(series.genre_ids, [1]);
        assert_eq!(series.rating_average, Some(9.0));
        let content = crate::repositories::content::list(&connection, 1).unwrap();
        assert_eq!(content.items[0].watch.session.watched_episodes, 3);
        let preferences = crate::services::preferences::access(temp.path(), None).unwrap();
        let preferences = serde_json::to_value(preferences).unwrap();
        assert_eq!(preferences["theme"], "light");
        assert_eq!(preferences["coverSize"], "large");
        assert_eq!(preferences["genreIds"][0], 1);
        assert_eq!(preferences["contentFilters"]["1"]["categoryId"], 1);
        assert_eq!(preferences["contentFilters"]["1"]["status"], "watching");
        assert!(
            !connection
                .prepare("PRAGMA foreign_key_check")
                .unwrap()
                .exists([])
                .unwrap()
        );
    }

    #[test]
    fn upgrades_version_three_with_one_initial_session_per_content() {
        let temp = tempdir().unwrap();
        let connection = Connection::open(temp.path().join(DATABASE_NAME)).unwrap();
        for (_, sql) in &MIGRATIONS[..3] {
            connection.execute_batch(sql).unwrap();
        }
        connection.execute_batch(
            "PRAGMA user_version = 3;
             INSERT INTO series (title, normalized_title) VALUES ('Serie', 'serie');
             INSERT INTO categories (name, normalized_name) VALUES ('Corto', 'corto');
             INSERT INTO contents (series_id, category_id, title, cover_key, description, opinion, release_year, sort_order)
             VALUES (1, 1, 'Segundo', 'existing', 'Descripción', 'Opinión', 2024, 5), (1, 1, 'Primero', NULL, '', '', NULL, 0);",
        ).unwrap();
        drop(connection);
        let connection = open(temp.path()).unwrap();
        let items = crate::repositories::content::list(&connection, 1)
            .unwrap()
            .items;
        assert_eq!(items[0].title, "Primero");
        assert_eq!(items[1].description, "Descripción");
        assert_eq!(items[1].opinion, "Opinión");
        assert_eq!(items[1].cover_key.as_deref(), Some("existing"));
        assert_eq!(items[1].release_year, Some(2024));
        for item in &items {
            assert_eq!(
                item.watch.session.status,
                crate::domain::watch::Status::ToWatch
            );
            assert!(item.watch.session.active);
            assert_eq!(item.watch.session.watched_episodes, 0);
            assert_eq!(item.watch.completed_views, 0);
            assert!(item.watch.total_episodes.is_none() && item.watch.rating_tenths.is_none());
        }
        drop(connection);
        let reopened = open(temp.path()).unwrap();
        let count: i64 = reopened
            .query_row("SELECT COUNT(*) FROM watch_sessions", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn upgrades_version_two_preserving_catalog_and_favorites() {
        let temp = tempdir().unwrap();
        let connection = Connection::open(temp.path().join(DATABASE_NAME)).unwrap();
        for (_, sql) in &MIGRATIONS[..2] {
            connection.execute_batch(sql).unwrap();
        }
        connection.execute_batch(
            "PRAGMA user_version = 2;
             INSERT INTO series (title, normalized_title, cover_key, favorite) VALUES ('Álbum', 'album', 'old-cover', 1);
             INSERT INTO genres (name, normalized_name) VALUES ('Drama', 'drama');
             INSERT INTO series_genres VALUES (1, 1);",
        ).unwrap();
        drop(connection);
        let connection = open(temp.path()).unwrap();
        assert_eq!(schema_version(&connection).unwrap(), 6);
        let catalog = crate::repositories::catalog(&connection).unwrap();
        let series = &catalog.series[0];
        assert_eq!(series.title, "Álbum");
        assert_eq!(series.cover_key.as_deref(), Some("old-cover"));
        assert!(series.favorite);
        assert_eq!(series.genre_ids, [catalog.genres[0].id]);
        assert!(
            crate::repositories::content::list(&connection, series.id)
                .unwrap()
                .categories
                .is_empty()
        );
    }

    #[test]
    fn upgrades_version_one_without_changing_settings() {
        let temporary = tempdir().unwrap();
        let connection = Connection::open(temporary.path().join(DATABASE_NAME)).unwrap();
        connection.execute_batch(MIGRATIONS[0].1).unwrap();
        connection.pragma_update(None, "user_version", 1).unwrap();
        connection
            .execute(
                "INSERT INTO settings VALUES (?1, ?2)",
                ["theme", "\"dark\""],
            )
            .unwrap();
        drop(connection);
        let connection = open(temporary.path()).unwrap();
        assert_eq!(schema_version(&connection).unwrap(), 6);
        let value: String = connection
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                ["theme"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(value, "\"dark\"");
        assert!(connection.prepare("SELECT id FROM series").is_ok());
    }

    #[test]
    fn initializes_and_preserves_data_on_reopen() {
        let temporary = tempdir().unwrap();
        let data_dir = temporary.path().join("app-data");
        let connection = open(&data_dir).unwrap();
        assert_eq!(schema_version(&connection).unwrap(), 6);
        connection
            .execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)",
                ["test", r#"{"enabled":true}"#],
            )
            .unwrap();
        drop(connection);

        let reopened = open(&data_dir).unwrap();
        assert_eq!(schema_version(&reopened).unwrap(), 6);
        assert!(
            reopened
                .pragma_query_value(None, "foreign_keys", |row| row.get::<_, bool>(0))
                .unwrap()
        );
        let value: String = reopened
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                ["test"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(value, r#"{"enabled":true}"#);
    }

    #[test]
    fn rejects_invalid_settings_values() {
        let temporary = tempdir().unwrap();
        let connection = open(temporary.path()).unwrap();
        let result = connection.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)",
            ["test", "invalid-json"],
        );
        assert_eq!(
            result.unwrap_err().sqlite_error_code(),
            Some(rusqlite::ErrorCode::ConstraintViolation)
        );
    }

    #[test]
    fn leaves_version_unchanged_when_migration_fails() {
        let temporary = tempdir().unwrap();
        let connection = Connection::open(temporary.path().join(DATABASE_NAME)).unwrap();
        connection
            .execute_batch("CREATE TABLE settings (old_value TEXT);")
            .unwrap();
        drop(connection);

        assert_eq!(open(temporary.path()).unwrap_err(), DatabaseError::Migrate);
        let unchanged = Connection::open(temporary.path().join(DATABASE_NAME)).unwrap();
        assert_eq!(schema_version(&unchanged).unwrap(), 0);
        assert!(unchanged.prepare("SELECT old_value FROM settings").is_ok());
    }

    #[test]
    fn rejects_a_newer_schema_without_downgrading_it() {
        let temporary = tempdir().unwrap();
        let connection = open(temporary.path()).unwrap();
        connection.pragma_update(None, "user_version", 99).unwrap();
        drop(connection);

        assert_eq!(
            open(temporary.path()).unwrap_err(),
            DatabaseError::UnsupportedSchema
        );
        let unchanged = Connection::open(temporary.path().join(DATABASE_NAME)).unwrap();
        assert_eq!(schema_version(&unchanged).unwrap(), 99);
    }

    #[test]
    fn reports_an_unusable_data_directory() {
        let temporary = tempdir().unwrap();
        let file = temporary.path().join("app-data");
        fs::write(&file, "file").unwrap();
        assert_eq!(open(&file).unwrap_err(), DatabaseError::CreateDirectory);
    }
}
