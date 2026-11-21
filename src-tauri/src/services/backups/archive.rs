use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tempfile::{NamedTempFile, TempDir};
use zip::{CompressionMethod, ZipArchive, ZipWriter, write::SimpleFileOptions};

use crate::infrastructure::{
    covers::{managed_root, valid_key},
    database::{self, DATABASE_NAME},
};

use super::{BackupError, Result};

const MANIFEST: &str = "manifest.json";
const FILE_LIMIT: usize = 100_000;
const MANIFEST_LIMIT: u64 = 32 * 1024 * 1024;
const ARCHIVE_LIMIT: u64 = 64 * 1024 * 1024 * 1024;
const DATABASE_LIMIT: u64 = 8 * 1024 * 1024 * 1024;
const IMAGE_LIMIT: u64 = 12 * 1024 * 1024;
const ASSET_FILES: [&str; 2] = ["original", "thumbnail.webp"];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Manual,
    Automatic,
    Preventive,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub created_at: DateTime<Utc>,
    pub schema_version: i32,
    pub kind: Kind,
    pub series_count: i64,
    pub content_count: i64,
    pub image_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    application: String,
    format_version: u32,
    #[serde(flatten)]
    summary: Summary,
    files: BTreeMap<String, Record>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Record {
    size: u64,
    sha256: String,
}

pub struct Prepared {
    pub directory: TempDir,
    pub summary: Summary,
}

fn allowed_size(name: &str) -> Result<u64> {
    match name {
        DATABASE_NAME => Ok(DATABASE_LIMIT),
        MANIFEST => Ok(MANIFEST_LIMIT),
        other => {
            let mut parts = other.split('/');
            match (parts.next(), parts.next(), parts.next(), parts.next()) {
                (Some("covers"), Some(key), Some(file), None)
                    if valid_key(key) && ASSET_FILES.contains(&file) =>
                {
                    Ok(IMAGE_LIMIT)
                }
                _ => Err(BackupError::UnsafePath),
            }
        }
    }
}

fn transfer(input: impl Read, mut output: impl Write, limit: u64) -> io::Result<Record> {
    let mut input = input.take(limit + 1);
    let mut hasher = Sha256::new();
    let mut buffer = [0; 64 * 1024];
    let mut size = 0;
    loop {
        let n = input.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        size += n as u64;
        if size > limit {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "entry size"));
        }
        hasher.update(&buffer[..n]);
        output.write_all(&buffer[..n])?;
    }
    let digits = b"0123456789abcdef";
    let mut sha256 = String::with_capacity(64);
    for byte in hasher.finalize() {
        sha256.push(char::from(digits[usize::from(byte >> 4)]));
        sha256.push(char::from(digits[usize::from(byte & 15)]));
    }
    Ok(Record { size, sha256 })
}

pub fn asset_keys(connection: &Connection) -> Result<BTreeSet<String>> {
    let keys = connection
        .prepare(
            "SELECT cover_key FROM series WHERE cover_key IS NOT NULL
         UNION SELECT panorama_key FROM series WHERE panorama_key IS NOT NULL
         UNION SELECT cover_key FROM contents WHERE cover_key IS NOT NULL",
        )?
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<rusqlite::Result<BTreeSet<_>>>()?;
    if keys.iter().any(|key| !valid_key(key)) {
        return Err(BackupError::InvalidDatabase);
    }
    Ok(keys)
}

fn asset_names(keys: &BTreeSet<String>) -> impl Iterator<Item = String> {
    keys.iter()
        .flat_map(|key| ASSET_FILES.map(|name| format!("covers/{key}/{name}")))
}

fn snapshot_sources(
    root: &Path,
    snapshot: &Path,
    keys: &BTreeSet<String>,
) -> Result<Vec<(String, PathBuf)>> {
    let mut sources = vec![(DATABASE_NAME.into(), snapshot.into())];
    if !keys.is_empty() {
        let covers = managed_root(root).ok_or(BackupError::MissingImage)?;
        for name in asset_names(keys) {
            let path = covers.join(
                name.strip_prefix("covers/")
                    .ok_or(BackupError::UnsafePath)?,
            );
            if path.canonicalize().ok().as_ref() != Some(&path) || !path.is_file() {
                return Err(BackupError::MissingImage);
            }
            sources.push((name, path));
        }
    }
    Ok(sources)
}

fn counts(connection: &Connection) -> Result<(i64, i64)> {
    Ok(connection.query_row(
        "SELECT (SELECT COUNT(*) FROM series), (SELECT COUNT(*) FROM contents)",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?)
}

pub fn create(root: &Path, target: &Path, kind: Kind, now: DateTime<Utc>) -> Result<Summary> {
    let connection = database::open(root)?;
    let scratch = TempDir::new_in(root)?;
    let snapshot = scratch.path().join(DATABASE_NAME);
    connection.execute(
        "VACUUM INTO ?1",
        [snapshot.to_str().ok_or(BackupError::Io)?],
    )?;
    let source = database::open(scratch.path())?;
    let keys = asset_keys(&source)?;
    let (series_count, content_count) = counts(&source)?;
    let summary = Summary {
        created_at: now,
        schema_version: database::schema_version(&source)?,
        kind,
        series_count,
        content_count,
        image_count: keys.len(),
    };
    let sources = snapshot_sources(root, &snapshot, &keys)?;
    if sources.len() >= FILE_LIMIT {
        return Err(BackupError::ArchiveLimit);
    }
    let mut stage = NamedTempFile::new_in(target.parent().ok_or(BackupError::Io)?)?;
    let mut writer = ZipWriter::new(stage.as_file_mut());
    let mut files = BTreeMap::new();
    let mut total = 0;
    for (name, path) in sources {
        let input = File::open(path)?;
        let limit = allowed_size(&name)?;
        let size = input.metadata()?.len();
        total += size;
        if size > limit || total > ARCHIVE_LIMIT {
            return Err(BackupError::ArchiveLimit);
        }
        let method = if name == DATABASE_NAME {
            CompressionMethod::Deflated
        } else {
            CompressionMethod::Stored
        };
        writer.start_file(
            &name,
            SimpleFileOptions::default()
                .compression_method(method)
                .large_file(size >= u64::from(u32::MAX)),
        )?;
        files.insert(name, transfer(input, &mut writer, limit)?);
    }
    writer.start_file(MANIFEST, SimpleFileOptions::default())?;
    serde_json::to_writer(
        &mut writer,
        &Manifest {
            application: "MyList".into(),
            format_version: 1,
            summary: summary.clone(),
            files,
        },
    )
    .map_err(|_| BackupError::Io)?;
    writer.finish()?.sync_all()?;
    let mut archive = read_zip(stage.reopen()?)?;
    let manifest = read_manifest(&mut archive)?;
    verify(&mut archive, &manifest, None)?;
    stage.persist(target).map_err(|_| BackupError::Io)?;
    Ok(summary)
}

fn read_zip(file: File) -> Result<ZipArchive<File>> {
    if file.metadata()?.len() > ARCHIVE_LIMIT + MANIFEST_LIMIT {
        return Err(BackupError::ArchiveLimit);
    }
    let mut archive = ZipArchive::new(file)?;
    if archive.len() > FILE_LIMIT {
        return Err(BackupError::ArchiveLimit);
    }
    archive.validate_header_consistency()?;
    Ok(archive)
}

fn read_manifest(archive: &mut ZipArchive<File>) -> Result<Manifest> {
    let mut entry = archive.by_name(MANIFEST)?;
    if entry.size() > MANIFEST_LIMIT || !entry.is_file() || entry.is_symlink() {
        return Err(BackupError::InvalidArchive);
    }
    let mut bytes = Vec::new();
    (&mut entry)
        .take(MANIFEST_LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| BackupError::InvalidArchive)?;
    if bytes.len() as u64 > MANIFEST_LIMIT {
        return Err(BackupError::ArchiveLimit);
    }
    let manifest: Manifest =
        serde_json::from_slice(&bytes).map_err(|_| BackupError::InvalidArchive)?;
    if manifest.application != "MyList"
        || manifest.format_version != 1
        || !(1..=database::latest_schema()).contains(&manifest.summary.schema_version)
    {
        return Err(BackupError::Incompatible);
    }
    Ok(manifest)
}

pub fn summary(path: &Path) -> Result<Summary> {
    Ok(read_manifest(&mut read_zip(File::open(path)?)?)?.summary)
}

fn verify(
    archive: &mut ZipArchive<File>,
    manifest: &Manifest,
    destination: Option<&Path>,
) -> Result<()> {
    if !manifest.files.contains_key(DATABASE_NAME) || archive.len() != manifest.files.len() + 1 {
        return Err(BackupError::InvalidArchive);
    }
    let mut seen = BTreeSet::new();
    let mut total = 0;
    for index in 0..archive.len() {
        let entry = archive.by_index(index)?;
        let name = entry.name()?.into_owned();
        let limit = allowed_size(&name)?;
        if !seen.insert(name.clone()) || !entry.is_file() || entry.is_symlink() {
            return Err(BackupError::UnsafePath);
        }
        if entry.size() > limit || entry.size() > ARCHIVE_LIMIT - total {
            return Err(BackupError::ArchiveLimit);
        }
        total += entry.size();
        if name == MANIFEST {
            continue;
        }
        let record = manifest
            .files
            .get(&name)
            .ok_or(BackupError::InvalidArchive)?;
        if record.size != entry.size() {
            return Err(BackupError::InvalidArchive);
        }
        let result = if let Some(root) = destination {
            let path = root.join(&name);
            fs::create_dir_all(path.parent().ok_or(BackupError::UnsafePath)?)?;
            transfer(entry, File::create_new(path)?, record.size)
        } else {
            transfer(entry, io::sink(), record.size)
        }
        .map_err(|error| {
            if error.kind() == io::ErrorKind::StorageFull {
                BackupError::Io
            } else {
                BackupError::InvalidArchive
            }
        })?;
        if result != *record {
            return Err(BackupError::InvalidArchive);
        }
    }
    Ok(())
}

type Schema = Vec<(String, String, String, String)>;

fn schema(connection: &Connection) -> rusqlite::Result<Schema> {
    connection
        .prepare(
            "SELECT type,name,tbl_name,sql FROM sqlite_schema
         WHERE name NOT GLOB 'sqlite_*' ORDER BY type,name",
        )?
        .query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?
        .collect()
}

fn validate_database(path: &Path, version: i32) -> Result<()> {
    let database = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    database.pragma_update(None, "trusted_schema", false)?;
    let mut baseline = Connection::open_in_memory()?;
    database::migrate_to(&mut baseline, version)?;
    if database::schema_version(&database)? != version
        || schema(&database)? != schema(&baseline)?
        || database.query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))? != "ok"
        || database.prepare("PRAGMA foreign_key_check")?.exists([])?
    {
        return Err(BackupError::InvalidDatabase);
    }
    Ok(())
}

pub fn prepare(root: &Path, path: &Path) -> Result<Prepared> {
    let mut archive = read_zip(File::open(path)?)?;
    let manifest = read_manifest(&mut archive)?;
    let directory = TempDir::new_in(root)?;
    verify(&mut archive, &manifest, Some(directory.path()))?;
    validate_database(
        &directory.path().join(DATABASE_NAME),
        manifest.summary.schema_version,
    )
    .map_err(|_| BackupError::InvalidDatabase)?;
    let connection = database::open(directory.path())?;
    super::super::preferences::access(directory.path(), None)
        .map_err(|_| BackupError::InvalidDatabase)?;
    let keys = asset_keys(&connection)?;
    let expected: BTreeSet<_> = asset_names(&keys).chain([DATABASE_NAME.into()]).collect();
    let (series, contents) = counts(&connection)?;
    if !expected.iter().eq(manifest.files.keys())
        || keys.len() != manifest.summary.image_count
        || (series, contents)
            != (
                manifest.summary.series_count,
                manifest.summary.content_count,
            )
        || connection
            .prepare(
                "SELECT 1 FROM contents c WHERE NOT EXISTS (
           SELECT 1 FROM watch_sessions w WHERE w.content_id = c.id AND w.active = 1
         ) LIMIT 1",
            )?
            .exists([])?
    {
        return Err(BackupError::InvalidDatabase);
    }
    Ok(Prepared {
        directory,
        summary: manifest.summary,
    })
}
