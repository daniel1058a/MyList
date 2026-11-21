use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::{State, WebviewWindow};
use uuid::Uuid;

use crate::services::{
    backups::{self, BackupError, Kind, Prepared, Result, Status, Summary},
    exports::{self, Format},
    preferences::Language,
};

use super::StorageLocation;

#[derive(Default)]
pub struct RestorePreview(Arc<Mutex<Option<(String, Prepared)>>>);

#[derive(Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum Action {
    Status,
    CheckAutomatic,
    SetAutomatic { enabled: bool },
    Save,
    OpenFolder,
    Preview,
    Cancel { token: String },
    Restore { token: String },
    Export { format: Format },
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Outcome {
    Status { status: Status },
    Saved { name: String },
    Preview { token: String, summary: Summary },
    Restored,
    Cancelled,
    Opened,
}

#[derive(Serialize)]
pub struct IpcError {
    code: BackupError,
}

async fn blocking<T: Send + 'static>(
    task: impl FnOnce() -> Result<T> + Send + 'static,
) -> Result<T> {
    tauri::async_runtime::spawn_blocking(task)
        .await
        .map_err(|_| BackupError::Io)?
}

#[cfg(windows)]
fn choose(window: &WebviewWindow, action: &Action, language: Language) -> Option<PathBuf> {
    let dialog = rfd::FileDialog::new().set_parent(window);
    match action {
        Action::Preview => dialog
            .set_title(match language {
                Language::Es => "Elegir respaldo de MyList",
                Language::En => "Choose a MyList backup",
            })
            .add_filter("ZIP", &["zip"])
            .pick_file(),
        Action::Save => dialog
            .set_title(match language {
                Language::Es => "Guardar respaldo de MyList",
                Language::En => "Save a MyList backup",
            })
            .add_filter("ZIP", &["zip"])
            .set_file_name(format!("MyList-{}.zip", Utc::now().format("%Y-%m-%d")))
            .save_file(),
        Action::Export { format } => {
            let extension = format.extension();
            dialog
                .set_title(match language {
                    Language::Es => "Exportar catálogo",
                    Language::En => "Export catalog",
                })
                .add_filter(extension.to_ascii_uppercase(), &[extension])
                .set_file_name(format!("MyList.{extension}"))
                .save_file()
        }
        _ => None,
    }
}

fn filename(path: &Path) -> Result<String> {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .ok_or(BackupError::InvalidDestination)
}

#[tauri::command]
pub async fn backup_access(
    window: WebviewWindow,
    location: State<'_, StorageLocation>,
    preview: State<'_, RestorePreview>,
    action: Action,
    language: Option<Language>,
) -> std::result::Result<Outcome, IpcError> {
    access(
        window,
        &location,
        &preview,
        action,
        language.unwrap_or_default(),
    )
    .await
    .map_err(|code| IpcError { code })
}

async fn access(
    window: WebviewWindow,
    location: &StorageLocation,
    preview: &RestorePreview,
    action: Action,
    language: Language,
) -> Result<Outcome> {
    let root = location.root.clone().ok_or(BackupError::Io)?;
    let gate = location.gate.clone();
    let preview = preview.0.clone();
    blocking(move || {
        let path: Option<PathBuf> = if matches!(
            action,
            Action::Save | Action::Preview | Action::Export { .. }
        ) {
            #[cfg(windows)]
            {
                let Some(path) = choose(&window, &action, language) else {
                    return Ok(Outcome::Cancelled);
                };
                Some(path)
            }
            #[cfg(not(windows))]
            {
                let _ = (window, language);
                return Err(BackupError::Io);
            }
        } else {
            None
        };
        let selected = || path.as_deref().ok_or(BackupError::InvalidDestination);
        let _operation = gate.write().map_err(|_| BackupError::Io)?;
        match action {
            Action::Status => Ok(Outcome::Status {
                status: backups::status(&root, None)?,
            }),
            Action::CheckAutomatic => Ok(Outcome::Status {
                status: backups::automatic(&root, Utc::now())?,
            }),
            Action::SetAutomatic { enabled } => Ok(Outcome::Status {
                status: backups::status(&root, Some(enabled))?,
            }),
            Action::Save => {
                let path = selected()?;
                backups::verify_destination(path, "zip")?;
                backups::create(&root, path, Kind::Manual, Utc::now())?;
                Ok(Outcome::Saved {
                    name: filename(path)?,
                })
            }
            Action::Preview => {
                let path = selected()?;
                let mut current = preview.lock().map_err(|_| BackupError::Io)?;
                *current = None;
                let data = backups::prepare(&root, path)?;
                let token = Uuid::new_v4().to_string();
                let summary = data.summary.clone();
                *current = Some((token.clone(), data));
                Ok(Outcome::Preview { token, summary })
            }
            Action::Cancel { token } => {
                let mut current = preview.lock().map_err(|_| BackupError::Io)?;
                if current
                    .as_ref()
                    .is_some_and(|(selected, _)| *selected == token)
                {
                    *current = None;
                }
                Ok(Outcome::Cancelled)
            }
            Action::Restore { token } => {
                let mut current = preview.lock().map_err(|_| BackupError::Io)?;
                if !current
                    .as_ref()
                    .is_some_and(|(selected, _)| *selected == token)
                {
                    return Err(BackupError::NoPreview);
                }
                let (_, data) = current.take().ok_or(BackupError::NoPreview)?;
                backups::restore(&root, data)?;
                Ok(Outcome::Restored)
            }
            Action::Export { format } => {
                let path = selected()?;
                exports::export(&root, path, format)?;
                Ok(Outcome::Saved {
                    name: filename(path)?,
                })
            }
            Action::OpenFolder => {
                #[cfg(windows)]
                {
                    backups::folder(&root, "automatic")?;
                    std::process::Command::new("explorer.exe")
                        .arg(root.join("backups"))
                        .spawn()?;
                    Ok(Outcome::Opened)
                }
                #[cfg(not(windows))]
                {
                    Err(BackupError::Io)
                }
            }
        }
    })
    .await
}
