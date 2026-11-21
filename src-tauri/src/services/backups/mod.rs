mod archive;

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Duration, Utc};
use rusqlite::{
    Connection, OptionalExtension,
    backup::{Backup, StepResult},
};
use serde::Serialize;
use uuid::Uuid;

use crate::infrastructure::{covers, database};

pub use archive::{Kind, Prepared, Summary, create, prepare};
pub type Result<T> = std::result::Result<T, BackupError>;

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BackupError {
    Io,
    InvalidArchive,
    Incompatible,
    UnsafePath,
    ArchiveLimit,
    InvalidDatabase,
    MissingImage,
    NoPreview,
    InvalidDestination,
    RestoreFailed,
}

impl From<io::Error> for BackupError {
    fn from(_: io::Error) -> Self {
        Self::Io
    }
}

impl From<zip::result::ZipError> for BackupError {
    fn from(_: zip::result::ZipError) -> Self {
        Self::InvalidArchive
    }
}

impl From<rusqlite::Error> for BackupError {
    fn from(_: rusqlite::Error) -> Self {
        Self::InvalidDatabase
    }
}

impl From<database::DatabaseError> for BackupError {
    fn from(_: database::DatabaseError) -> Self {
        Self::InvalidDatabase
    }
}

const AUTOMATIC_KEY: &str = "backup.automatic.enabled";
const RETENTION: usize = 5;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    automatic: bool,
    last_success: Option<DateTime<Utc>>,
}

pub fn folder(root: &Path, name: &str) -> Result<PathBuf> {
    let path = root.canonicalize()?.join("backups").join(name);
    fs::create_dir_all(&path)?;
    if path.canonicalize()? != path {
        return Err(BackupError::UnsafePath);
    }
    Ok(path)
}

fn filename(prefix: &str, at: DateTime<Utc>) -> String {
    format!(
        "{prefix}-{}-{}.zip",
        at.format("%Y%m%dT%H%M%SZ"),
        Uuid::new_v4()
    )
}

fn automatic_files(root: &Path) -> Result<Vec<(DateTime<Utc>, PathBuf)>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(folder(root, "automatic")?)? {
        let entry = entry?;
        let Some(name) = entry.file_name().into_string().ok() else {
            continue;
        };
        let generated = name
            .strip_prefix("automatic-")
            .and_then(|name| name.strip_suffix(".zip"))
            .and_then(|name| name.split_once('-'))
            .is_some_and(|(date, id)| date.len() == 16 && covers::valid_key(id));
        if generated && entry.file_type()?.is_file() {
            let path = entry.path();
            if let Ok(summary) = archive::summary(&path)
                && summary.kind == Kind::Automatic
            {
                files.push((summary.created_at, path));
            }
        }
    }
    files.sort_unstable_by(|a, b| b.cmp(a));
    Ok(files)
}

pub fn status(root: &Path, enabled: Option<bool>) -> Result<Status> {
    let connection = database::open(root)?;
    if let Some(enabled) = enabled {
        connection.execute(
            "INSERT INTO settings(key,value) VALUES (?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            rusqlite::params![AUTOMATIC_KEY, enabled.to_string()],
        )?;
    }
    let disabled = connection
        .query_row(
            "SELECT value = 'false' FROM settings WHERE key=?1",
            [AUTOMATIC_KEY],
            |row| row.get::<_, bool>(0),
        )
        .optional()?
        .unwrap_or(false);
    let last_success = automatic_files(root)?.first().map(|(at, _)| *at);
    Ok(Status {
        automatic: !disabled,
        last_success,
    })
}

pub fn automatic(root: &Path, now: DateTime<Utc>) -> Result<Status> {
    let current = status(root, None)?;
    if !current.automatic
        || current
            .last_success
            .is_some_and(|previous| now - previous < Duration::days(1))
    {
        return Ok(current);
    }
    let destination = folder(root, "automatic")?.join(filename("automatic", now));
    create(root, &destination, Kind::Automatic, now)?;
    for (_, path) in automatic_files(root)?.into_iter().skip(RETENTION) {
        let _ = fs::remove_file(path);
    }
    Ok(Status {
        last_success: Some(now),
        ..current
    })
}

pub fn verify_destination(path: &Path, extension: &str) -> Result<()> {
    if path
        .extension()
        .and_then(|part| part.to_str())
        .is_some_and(|part| part.eq_ignore_ascii_case(extension))
        && path.is_absolute()
    {
        Ok(())
    } else {
        Err(BackupError::InvalidDestination)
    }
}

fn replace_database(source: &Connection, destination: &mut Connection) -> Result<()> {
    let operation = Backup::new(source, destination)?;
    if operation.step(-1)? == StepResult::Done {
        Ok(())
    } else {
        Err(BackupError::RestoreFailed)
    }
}

pub fn restore(root: &Path, prepared: Prepared) -> Result<()> {
    restore_with(root, prepared, replace_database)
}

fn restore_with(
    root: &Path,
    prepared: Prepared,
    apply: impl FnOnce(&Connection, &mut Connection) -> Result<()>,
) -> Result<()> {
    let now = Utc::now();
    let preventive = folder(root, "preventive")?.join(filename("preventive", now));
    create(root, &preventive, Kind::Preventive, now)?;

    let mut current = database::open(root)?;
    let old_keys = archive::asset_keys(&current)?;
    let source = database::open(prepared.directory.path())?;
    let restored_keys = archive::asset_keys(&source)?;
    let mut installed = Vec::new();
    let result = (|| -> Result<()> {
        fs::create_dir_all(root.join("covers"))?;
        let destination = covers::managed_root(root).ok_or(BackupError::UnsafePath)?;
        for key in restored_keys {
            let fresh = Uuid::new_v4().to_string();
            let staged = prepared.directory.path().join("covers").join(&key);
            for name in ["original", "thumbnail.webp"] {
                fs::OpenOptions::new()
                    .write(true)
                    .open(staged.join(name))?
                    .sync_all()?;
            }
            fs::rename(staged, destination.join(&fresh))?;
            installed.push(fresh.clone());
            for sql in [
                "UPDATE series SET cover_key=?1 WHERE cover_key=?2",
                "UPDATE series SET panorama_key=?1 WHERE panorama_key=?2",
                "UPDATE contents SET cover_key=?1 WHERE cover_key=?2",
            ] {
                source.execute(sql, [&fresh, &key])?;
            }
        }
        apply(&source, &mut current)
    })();
    if result.is_ok() {
        for key in old_keys {
            covers::remove(root, &key);
        }
    } else {
        for key in installed {
            covers::remove(root, &key);
        }
    }
    result.map_err(|_| BackupError::RestoreFailed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{Engine, engine::general_purpose::STANDARD};
    use image::{DynamicImage, ImageFormat};
    use std::io::{Cursor, Write};
    use tempfile::tempdir;
    use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

    fn asset(root: &Path, panorama: bool) -> String {
        let mut bytes = Cursor::new(Vec::new());
        DynamicImage::new_rgb8(16, 9)
            .write_to(&mut bytes, ImageFormat::Png)
            .unwrap();
        let replacement = covers::CoverChange::Replace {
            data: STANDARD.encode(bytes.into_inner()),
        };
        let prepared = if panorama {
            replacement.prepare_panorama()
        } else {
            replacement.prepare()
        }
        .unwrap();
        prepared
            .persist::<_, covers::CoverError>(root, None, |key| Ok(key.unwrap().to_owned()))
            .unwrap()
    }

    #[test]
    fn restores_wal_snapshot_images_relationships_and_settings_with_a_preventive_copy() {
        let temporary = tempdir().unwrap();
        let source = temporary.path().join("source");
        let destination = temporary.path().join("destination");
        let live = database::open(&source).unwrap();
        live.pragma_update(None, "journal_mode", "WAL").unwrap();
        live.pragma_update(None, "wal_autocheckpoint", 0).unwrap();
        let [poster, backdrop, entry] = [
            asset(&source, false),
            asset(&source, true),
            asset(&source, false),
        ];
        live.execute(
            "INSERT INTO series(title,normalized_title,cover_key,panorama_key,favorite,description)
             VALUES ('Órbita','orbita',?1,?2,1,'Descripción')",
            [&poster, &backdrop],
        )
        .unwrap();
        live.execute_batch(
            "INSERT INTO genres(name,normalized_name) VALUES ('Drama','drama');
             INSERT INTO series_genres VALUES (1,1);
             INSERT INTO categories(name,normalized_name) VALUES ('Corto','corto');",
        )
        .unwrap();
        live.execute(
            "INSERT INTO contents(series_id,category_id,title,sort_order,cover_key,rating_tenths,total_episodes)
             VALUES (1,1,'Piloto',0,?1,95,3)", [&entry],
        ).unwrap();
        live.execute_batch(
            r#"UPDATE watch_sessions SET active=0,status='finished',completed=1,watched_episodes=3,started_on='2025-05-01';
             INSERT INTO watch_sessions(content_id,watched_episodes,status) VALUES (1,1,'watching');
             INSERT INTO settings VALUES ('catalog.preferences.v1','{"language":"en","theme":"light"}');
             INSERT INTO settings VALUES ('backup.automatic.enabled','false');"#,
        ).unwrap();
        assert!(source.join("database.sqlite3-wal").exists());
        let path = temporary.path().join("manual.zip");
        let summary = create(&source, &path, Kind::Manual, Utc::now()).unwrap();
        assert_eq!(
            (
                summary.series_count,
                summary.content_count,
                summary.image_count
            ),
            (1, 1, 3)
        );
        let zip = ZipArchive::new(fs::File::open(&path).unwrap()).unwrap();
        assert_eq!(zip.len(), 8);
        assert!(
            !zip.file_names()
                .map(|name| name.unwrap())
                .any(|name| name.ends_with("-wal") || name.ends_with("-shm"))
        );
        database::open(&destination)
            .unwrap()
            .execute(
                "INSERT INTO series(title,normalized_title) VALUES ('Anterior','anterior')",
                [],
            )
            .unwrap();

        let preview = prepare(&destination, &path).unwrap();
        assert_eq!(preview.summary, summary);
        restore(&destination, preview).unwrap();
        let reopened = database::open(&destination).unwrap();
        let catalog = crate::repositories::catalog(&reopened).unwrap();
        let series = &catalog.series[0];
        assert_eq!(series.title, "Órbita");
        assert_eq!(series.genre_ids, [1]);
        assert_eq!(series.rating_average, Some(9.5));
        assert!(series.favorite);
        let item = crate::repositories::content::list(&reopened, series.id)
            .unwrap()
            .items
            .remove(0);
        assert_eq!(item.category_id, 1);
        assert_eq!(item.watch.completed_views, 1);
        assert_eq!(item.watch.session.watched_episodes, 1);
        for (original, restored) in [
            (&poster, series.cover_key.as_ref().unwrap()),
            (&backdrop, series.panorama_key.as_ref().unwrap()),
            (&entry, item.cover_key.as_ref().unwrap()),
        ] {
            assert_ne!(original, restored);
            assert_eq!(
                covers::thumbnail(&source, original),
                covers::thumbnail(&destination, restored)
            );
        }
        assert!(!status(&destination, None).unwrap().automatic);
        let value = super::super::preferences::access(&destination, None).unwrap();
        assert_eq!(value.language, super::super::preferences::Language::En);
        let preventive = fs::read_dir(folder(&destination, "preventive").unwrap())
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        assert_eq!(
            prepare(&destination, &preventive).unwrap().summary.kind,
            Kind::Preventive
        );
    }

    #[test]
    fn interrupted_sqlite_restore_leaves_previous_rows_and_images_available() {
        let temporary = tempdir().unwrap();
        let source = temporary.path().join("replacement");
        let destination = temporary.path().join("current");
        for (root, title) in [(&source, "Nueva"), (&destination, "Anterior")] {
            let storage = database::open(root).unwrap();
            let key = asset(root, false);
            storage
                .execute(
                    "INSERT INTO series(title,normalized_title,cover_key) VALUES (?1,?1,?2)",
                    [title, &key],
                )
                .unwrap();
        }
        let before = archive::asset_keys(&database::open(&destination).unwrap()).unwrap();
        let zip = temporary.path().join("source.zip");
        create(&source, &zip, Kind::Manual, Utc::now()).unwrap();
        let staged = prepare(&destination, &zip).unwrap();
        assert_eq!(
            restore_with(&destination, staged, |from, to| {
                let op = Backup::new(from, to).unwrap();
                assert_eq!(op.step(1).unwrap(), StepResult::More);
                Err(BackupError::Io)
            }),
            Err(BackupError::RestoreFailed)
        );
        let current = database::open(&destination).unwrap();
        assert_eq!(
            crate::repositories::catalog(&current).unwrap().series[0].title,
            "Anterior"
        );
        assert_eq!(archive::asset_keys(&current).unwrap(), before);
        assert_eq!(fs::read_dir(destination.join("covers")).unwrap().count(), 1);
        assert!(covers::thumbnail(&destination, before.first().unwrap()).is_some());
        assert_eq!(
            fs::read_dir(folder(&destination, "preventive").unwrap())
                .unwrap()
                .count(),
            1
        );
    }

    #[test]
    fn rejects_corrupt_digests_traversal_and_future_versions() {
        let temporary = tempdir().unwrap();
        let root = temporary.path();
        let source = root.join("original.zip");
        create(root, &source, Kind::Manual, Utc::now()).unwrap();
        let mut valid = ZipArchive::new(fs::File::open(&source).unwrap()).unwrap();
        let manifest: serde_json::Value =
            serde_json::from_reader(valid.by_name("manifest.json").unwrap()).unwrap();
        for (index, name, schema, error) in [
            (0, "database.sqlite3", 6, BackupError::InvalidArchive),
            (1, "../database.sqlite3", 6, BackupError::UnsafePath),
            (2, "database.sqlite3", 999, BackupError::Incompatible),
        ] {
            let target = root.join(format!("bad-{index}.zip"));
            let mut writer = ZipWriter::new(fs::File::create(&target).unwrap());
            writer
                .start_file(name, SimpleFileOptions::default())
                .unwrap();
            let mut altered = manifest.clone();
            altered["schemaVersion"] = schema.into();
            let size = altered["files"]["database.sqlite3"]["size"]
                .as_u64()
                .unwrap() as usize;
            writer.write_all(&vec![0; size]).unwrap();
            writer
                .start_file("manifest.json", SimpleFileOptions::default())
                .unwrap();
            serde_json::to_writer(&mut writer, &altered).unwrap();
            writer.finish().unwrap();
            assert!(matches!(prepare(root, &target), Err(found) if found == error));
        }
        let corrupt = root.join("truncated.zip");
        fs::write(&corrupt, b"PK\x03\x04").unwrap();
        assert!(matches!(
            prepare(root, &corrupt),
            Err(BackupError::InvalidArchive)
        ));
    }

    #[test]
    fn preserves_five_automatic_copies_and_manual_files_until_a_verified_write() {
        let temporary = tempdir().unwrap();
        let root = temporary.path();
        let start = Utc::now();
        database::open(root).unwrap();
        let manual = folder(root, "automatic")
            .unwrap()
            .join(filename("automatic", start));
        create(root, &manual, Kind::Manual, start).unwrap();
        for day in 0..7 {
            automatic(root, start + Duration::days(day)).unwrap();
        }
        let previous = automatic_files(root).unwrap();
        assert_eq!(previous.len(), 5);
        assert!(manual.exists());
        assert_eq!(
            automatic(root, start + Duration::days(6))
                .unwrap()
                .last_success,
            Some(start + Duration::days(6))
        );
        let nonexistent = Uuid::new_v4().to_string();
        database::open(root)
            .unwrap()
            .execute(
                "INSERT INTO series(title,normalized_title,cover_key) VALUES ('Serie','serie',?1)",
                [&nonexistent],
            )
            .unwrap();
        assert!(matches!(
            automatic(root, start + Duration::days(7)),
            Err(BackupError::MissingImage)
        ));
        assert_eq!(automatic_files(root).unwrap(), previous);
        status(root, Some(false)).unwrap();
        assert!(
            !automatic(root, start + Duration::days(7))
                .unwrap()
                .automatic
        );
    }
}
