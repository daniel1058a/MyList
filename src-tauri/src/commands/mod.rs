pub mod backups;
pub use backups::RestorePreview;

use std::{
    fs::{File, OpenOptions},
    path::PathBuf,
    sync::{Arc, RwLock},
};

use serde::Serialize;
use tauri::State;

use crate::{
    domain::watch::Session,
    repositories::{Catalog, content::SeriesContents},
    services::{
        catalog::{self, CatalogError, SaveSeries},
        content::{self, Placement, SaveContent},
        insights::{self, ContinueItem, Statistics},
        preferences::{self, Preferences},
        taxonomy::{self, Change, Taxonomy},
        watch::{self, WatchUpdate},
    },
};
use crate::{infrastructure::database::DatabaseError, services::health};

pub struct StorageLocation {
    pub root: Option<PathBuf>,
    pub gate: Arc<RwLock<()>>,
    _lease: Option<File>,
}

impl StorageLocation {
    pub fn new(mut root: Option<PathBuf>) -> Self {
        let lease = root.as_deref().and_then(|root| {
            std::fs::create_dir_all(root).ok()?;
            let lock = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(root.join("library.lock"))
                .ok()?;
            lock.try_lock().ok()?;
            Some(lock)
        });
        if lease.is_none() {
            root = None;
        }
        Self {
            root,
            gate: Arc::new(RwLock::new(())),
            _lease: lease,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageHealth {
    schema_version: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthError {
    DataDirectoryUnavailable,
    DatabaseUnavailable,
    SchemaUnreadable,
    SchemaUnsupported,
    MigrationFailed,
    DiagnosticFailed,
}

impl From<DatabaseError> for HealthError {
    fn from(error: DatabaseError) -> Self {
        match error {
            DatabaseError::CreateDirectory => Self::DataDirectoryUnavailable,
            DatabaseError::Open | DatabaseError::Configure => Self::DatabaseUnavailable,
            DatabaseError::ReadSchema => Self::SchemaUnreadable,
            DatabaseError::UnsupportedSchema => Self::SchemaUnsupported,
            DatabaseError::Migrate => Self::MigrationFailed,
        }
    }
}

#[derive(Serialize)]
pub struct IpcError {
    code: HealthError,
}

#[tauri::command]
pub async fn storage_health(
    location: State<'_, StorageLocation>,
) -> Result<StorageHealth, IpcError> {
    let check = async {
        let data_dir = location
            .root
            .clone()
            .ok_or(HealthError::DataDirectoryUnavailable)?;
        let gate = location.gate.clone();
        let schema_version = tauri::async_runtime::spawn_blocking(move || {
            let _operation = gate.read().map_err(|_| HealthError::DiagnosticFailed)?;
            health::check_storage(&data_dir).map_err(HealthError::from)
        })
        .await
        .map_err(|_| HealthError::DiagnosticFailed)??;
        Ok(StorageHealth { schema_version })
    };
    check.await.map_err(|code| IpcError { code })
}

#[derive(Serialize)]
pub struct CatalogIpcError {
    code: CatalogError,
}

async fn catalog_task<T: Send + 'static>(
    location: &StorageLocation,
    task: impl FnOnce(&std::path::Path) -> Result<T, CatalogError> + Send + 'static,
) -> Result<T, CatalogIpcError> {
    let unavailable = || CatalogIpcError {
        code: CatalogError::CatalogUnavailable,
    };
    let root = location.root.clone().ok_or_else(unavailable)?;
    let gate = location.gate.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _operation = gate.read().map_err(|_| CatalogError::CatalogUnavailable)?;
        task(&root)
    })
    .await
    .map_err(|_| unavailable())?
    .map_err(|code| CatalogIpcError { code })
}

#[tauri::command]
pub async fn library_statistics(
    location: State<'_, StorageLocation>,
) -> Result<Statistics, CatalogIpcError> {
    catalog_task(&location, insights::statistics).await
}

#[tauri::command]
pub async fn continue_watching(
    location: State<'_, StorageLocation>,
) -> Result<Vec<ContinueItem>, CatalogIpcError> {
    catalog_task(&location, insights::continue_watching).await
}

#[tauri::command]
pub async fn catalog_list(
    location: State<'_, StorageLocation>,
) -> Result<Catalog, CatalogIpcError> {
    catalog_task(&location, catalog::list).await
}

#[tauri::command]
pub async fn catalog_preferences(
    location: State<'_, StorageLocation>,
    value: Option<Preferences>,
) -> Result<Preferences, CatalogIpcError> {
    catalog_task(&location, move |root| preferences::access(root, value)).await
}

#[tauri::command]
pub async fn catalog_taxonomy(
    location: State<'_, StorageLocation>,
    change: Option<Change>,
) -> Result<Taxonomy, CatalogIpcError> {
    catalog_task(&location, move |root| taxonomy::access(root, change)).await
}

#[tauri::command]
pub async fn series_save(
    location: State<'_, StorageLocation>,
    input: SaveSeries,
) -> Result<Catalog, CatalogIpcError> {
    catalog_task(&location, move |root| catalog::save(root, input)).await
}

#[tauri::command]
pub async fn series_favorite(
    location: State<'_, StorageLocation>,
    id: i64,
    favorite: bool,
) -> Result<Catalog, CatalogIpcError> {
    catalog_task(&location, move |root| catalog::favorite(root, id, favorite)).await
}

#[tauri::command]
pub async fn series_delete(
    location: State<'_, StorageLocation>,
    id: i64,
) -> Result<Catalog, CatalogIpcError> {
    catalog_task(&location, move |root| catalog::delete(root, id)).await
}

#[tauri::command]
pub async fn content_list(
    location: State<'_, StorageLocation>,
    series_id: i64,
) -> Result<SeriesContents, CatalogIpcError> {
    catalog_task(&location, move |root| content::list(root, series_id)).await
}

#[tauri::command]
pub async fn content_save(
    location: State<'_, StorageLocation>,
    input: SaveContent,
) -> Result<SeriesContents, CatalogIpcError> {
    catalog_task(&location, move |root| content::save(root, input)).await
}

#[tauri::command]
pub async fn content_delete(
    location: State<'_, StorageLocation>,
    series_id: i64,
    id: i64,
) -> Result<SeriesContents, CatalogIpcError> {
    catalog_task(&location, move |root| content::delete(root, series_id, id)).await
}

#[tauri::command]
pub async fn content_reorder(
    location: State<'_, StorageLocation>,
    series_id: i64,
    id: i64,
    target_id: i64,
    placement: Placement,
) -> Result<SeriesContents, CatalogIpcError> {
    catalog_task(&location, move |root| {
        content::reorder(root, series_id, id, target_id, placement)
    })
    .await
}

#[tauri::command]
pub async fn watch_update(
    location: State<'_, StorageLocation>,
    input: WatchUpdate,
) -> Result<SeriesContents, CatalogIpcError> {
    catalog_task(&location, move |root| watch::update(root, input)).await
}

#[tauri::command]
pub async fn watch_history(
    location: State<'_, StorageLocation>,
    series_id: i64,
    content_id: i64,
) -> Result<Vec<Session>, CatalogIpcError> {
    catalog_task(&location, move |root| {
        watch::history(root, series_id, content_id)
    })
    .await
}
