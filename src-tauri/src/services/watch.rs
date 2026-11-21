use std::path::Path;

use rusqlite::{Error as SqlError, TransactionBehavior};
use serde::Deserialize;

use crate::{
    domain::watch::{Progress, Session, Status, WatchError, validate_dates, validate_rating},
    repositories::{
        content::{self as contents, SeriesContents},
        watch as repo,
    },
};

use super::{
    catalog::{CatalogError, open},
    content::finish,
};
type Result<T> = std::result::Result<T, CatalogError>;

impl From<WatchError> for CatalogError {
    fn from(error: WatchError) -> Self {
        match error {
            WatchError::InvalidProgress => Self::InvalidProgress,
            WatchError::InvalidRating => Self::InvalidRating,
            WatchError::InvalidWatchDates => Self::InvalidWatchDates,
            WatchError::ProgressResetRequired => Self::ProgressResetRequired,
            WatchError::RewatchRequiresCompletion => Self::RewatchRequiresCompletion,
        }
    }
}

#[derive(Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
enum Action {
    Step {
        delta: i32,
    },
    SetStatus {
        status: Status,
        reset_progress: bool,
    },
    Edit {
        progress: Option<Progress>,
        rating_tenths: Option<i64>,
        started_on: Option<String>,
        finished_on: Option<String>,
    },
    Rewatch,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchUpdate {
    series_id: i64,
    content_id: i64,
    session_id: i64,
    action: Action,
}

fn content_error(error: SqlError) -> CatalogError {
    match error {
        SqlError::QueryReturnedNoRows => CatalogError::ContentNotFound,
        _ => error.into(),
    }
}

pub fn history(data_dir: &Path, series_id: i64, content_id: i64) -> Result<Vec<Session>> {
    let mut connection = open(data_dir)?;
    let transaction = connection.transaction()?;
    contents::cover_key(&transaction, series_id, content_id).map_err(content_error)?;
    let history = repo::history(&transaction, content_id)?;
    transaction.commit()?;
    Ok(history)
}

pub fn update(data_dir: &Path, input: WatchUpdate) -> Result<SeriesContents> {
    let WatchUpdate {
        series_id,
        content_id,
        session_id,
        action,
    } = input;
    let mut connection = open(data_dir)?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let (mut session, mut total, mut rating) =
        repo::current(&transaction, series_id, content_id).map_err(content_error)?;
    if session.id != session_id {
        return Err(CatalogError::WatchSessionChanged);
    }

    let rewatch = matches!(action, Action::Rewatch);
    match action {
        Action::Step { delta } if matches!(delta, -1 | 1) => {
            session.apply_progress(&Progress {
                total_episodes: total,
                watched_episodes: session.watched_episodes + i64::from(delta),
            })?;
        }
        Action::Step { .. } => return Err(CatalogError::InvalidProgress),
        Action::SetStatus {
            status,
            reset_progress,
        } => {
            session.set_status(status, total, reset_progress)?;
        }
        Action::Edit {
            progress,
            rating_tenths,
            started_on,
            finished_on,
        } => {
            validate_rating(rating_tenths)?;
            validate_dates(started_on.as_deref(), finished_on.as_deref())?;
            if let Some(progress) = progress {
                session.apply_progress(&progress)?;
                total = progress.total_episodes;
            }
            session.started_on = started_on;
            session.finished_on = finished_on;
            rating = rating_tenths;
        }
        Action::Rewatch => {
            session.check_rewatch()?;
            repo::start_next(&transaction, content_id, session.id)?;
        }
    }
    if !rewatch {
        repo::save_session(&transaction, &session)?;
    }
    repo::save_content_watch(&transaction, content_id, total, rating)?;
    contents::touch_series(&transaction, series_id)?;
    finish(transaction, series_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn fixture(root: &Path) {
        open(root).unwrap().execute_batch(
            "INSERT INTO series (title, normalized_title) VALUES ('Serie', 'serie'), ('Otra', 'otra');
             INSERT INTO categories (name, normalized_name) VALUES ('Corto', 'corto');
             INSERT INTO contents (series_id, category_id, title, sort_order)
             VALUES (1, 1, 'Uno', 0), (1, 1, 'Dos', 1), (1, 1, 'Tres', 2),
                    (1, 1, 'Sin nota', 3), (2, 1, 'Otro', 0);",
        ).unwrap();
    }

    fn change(
        root: &Path,
        content_id: i64,
        session_id: i64,
        action: Action,
    ) -> Result<SeriesContents> {
        update(
            root,
            WatchUpdate {
                series_id: 1,
                content_id,
                session_id,
                action,
            },
        )
    }

    fn edit(progress: Option<(Option<i64>, i64)>, rating: Option<i64>) -> Action {
        Action::Edit {
            progress: progress.map(|(total_episodes, watched_episodes)| Progress {
                total_episodes,
                watched_episodes,
            }),
            rating_tenths: rating,
            started_on: None,
            finished_on: None,
        }
    }

    fn status(status: Status, reset_progress: bool) -> Action {
        Action::SetStatus {
            status,
            reset_progress,
        }
    }

    #[test]
    fn aggregates_only_rated_contents_and_updates_after_removal_and_deletion() {
        let temp = tempdir().unwrap();
        let root = temp.path();
        fixture(root);
        let catalog_average = || {
            super::super::catalog::list(root)
                .unwrap()
                .series
                .into_iter()
                .find(|series| series.id == 1)
                .unwrap()
                .rating_average
        };
        assert_eq!(catalog_average(), None);
        change(root, 1, 1, edit(None, Some(90))).unwrap();
        change(root, 2, 2, edit(None, Some(70))).unwrap();
        let snapshot = change(root, 3, 3, edit(None, Some(100))).unwrap();
        assert_eq!(snapshot.items[0].watch.session.status, Status::ToWatch);
        assert!((snapshot.rating_average.unwrap() - 26.0 / 3.0).abs() < 1e-10);
        assert_eq!(snapshot.rating_average, catalog_average());
        change(root, 1, 1, edit(None, Some(85))).unwrap();
        assert_eq!(
            change(root, 3, 3, edit(None, None)).unwrap().rating_average,
            Some(7.75)
        );
        assert_eq!(
            super::super::content::delete(root, 1, 2)
                .unwrap()
                .rating_average,
            Some(8.5)
        );
        assert_eq!(catalog_average(), Some(8.5));
        assert_eq!(
            change(root, 1, 1, edit(None, None)).unwrap().rating_average,
            None
        );
        assert_eq!(catalog_average(), None);
    }

    #[test]
    fn preserves_history_during_repeated_states_and_rejects_stale_session_updates() {
        let temp = tempdir().unwrap();
        let root = temp.path();
        fixture(root);
        assert_eq!(
            change(root, 1, 1, Action::Rewatch).unwrap_err(),
            CatalogError::RewatchRequiresCompletion
        );
        assert_eq!(
            change(root, 1, 1, Action::Step { delta: -1 }).unwrap_err(),
            CatalogError::InvalidProgress
        );
        change(root, 1, 1, edit(Some((Some(2), 1)), Some(91))).unwrap();
        assert_eq!(
            change(root, 1, 1, edit(Some((Some(0), 0)), None)).unwrap_err(),
            CatalogError::InvalidProgress
        );
        let complete = change(root, 1, 1, Action::Step { delta: 1 }).unwrap();
        assert_eq!(complete.items[0].watch.completed_views, 1);
        assert_eq!(
            change(root, 1, 1, Action::Step { delta: 1 }).unwrap_err(),
            CatalogError::InvalidProgress
        );

        change(root, 1, 1, status(Status::Watching, false)).unwrap();
        let dated = change(
            root,
            1,
            1,
            Action::Edit {
                progress: None,
                rating_tenths: Some(91),
                started_on: Some("2024-01-01".into()),
                finished_on: Some("2024-02-29".into()),
            },
        )
        .unwrap();
        assert_eq!(dated.items[0].watch.session.status, Status::Watching);
        let reopened = change(root, 1, 1, status(Status::Finished, false)).unwrap();
        assert_eq!(reopened.items[0].watch.completed_views, 1);
        assert_eq!(
            change(root, 1, 1, status(Status::ToWatch, false)).unwrap_err(),
            CatalogError::ProgressResetRequired
        );
        let reset = change(root, 1, 1, status(Status::ToWatch, true)).unwrap();
        assert_eq!(reset.items[0].watch.session.watched_episodes, 0);
        change(root, 1, 1, status(Status::Finished, false)).unwrap();

        let second = change(root, 1, 1, Action::Rewatch).unwrap();
        let watch = &second.items[0].watch;
        assert_eq!(watch.session.status, Status::ToWatch);
        assert_eq!(watch.session.watched_episodes, 0);
        assert!(watch.session.started_on.is_none());
        assert_eq!(watch.total_episodes, Some(2));
        assert_eq!(watch.rating_tenths, Some(91));
        assert_eq!(watch.completed_views, 1);
        assert_eq!(
            change(root, 1, 1, Action::Rewatch).unwrap_err(),
            CatalogError::WatchSessionChanged
        );
        change(root, 1, watch.session.id, status(Status::Finished, false)).unwrap();
        let sessions = history(root, 1, 1).unwrap();
        assert_eq!(sessions.len(), 2);
        assert!(sessions[0].active && !sessions[1].active);
        assert_eq!(sessions[1].finished_on.as_deref(), Some("2024-02-29"));
        assert!(sessions.iter().all(|session| session.completed));
        assert_eq!(
            super::super::content::list(root, 1).unwrap().items[0]
                .watch
                .completed_views,
            2
        );
        assert_eq!(
            history(root, 2, 1).unwrap_err(),
            CatalogError::ContentNotFound
        );
    }

    #[test]
    fn failed_rewatch_rolls_back_the_active_flag_and_deletion_cascades() {
        let temp = tempdir().unwrap();
        let root = temp.path();
        fixture(root);
        change(root, 1, 1, status(Status::Finished, false)).unwrap();
        let connection = open(root).unwrap();
        connection
            .execute_batch(
                "CREATE TRIGGER fail_session BEFORE INSERT ON watch_sessions
             BEGIN SELECT RAISE(ABORT, 'test'); END;",
            )
            .unwrap();
        assert_eq!(
            change(root, 1, 1, Action::Rewatch).unwrap_err(),
            CatalogError::CatalogUnavailable
        );
        let sessions = history(root, 1, 1).unwrap();
        assert_eq!(sessions.len(), 1);
        assert!(sessions[0].active && sessions[0].completed);
        super::super::content::delete(root, 1, 1).unwrap();
        assert!(repo::history(&connection, 1).unwrap().is_empty());
        super::super::catalog::delete(root, 1).unwrap();
        assert!(repo::history(&connection, 2).unwrap().is_empty());
        assert_eq!(repo::history(&connection, 5).unwrap().len(), 1);
    }
}
