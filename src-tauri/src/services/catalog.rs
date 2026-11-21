use std::{collections::BTreeMap, path::Path};

use rusqlite::{Error as SqlError, TransactionBehavior};
use serde::Deserialize;

use crate::{
    domain::{normalize, valid_label},
    infrastructure::{
        covers::{self, CoverChange, CoverError},
        database,
    },
    repositories::{self as repo, Catalog, SeriesValues},
};

#[derive(Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CatalogError {
    CatalogUnavailable,
    InvalidTitle,
    InvalidGenre,
    TooManyGenres,
    DuplicateTitle,
    SeriesNotFound,
    InvalidCover,
    CoverTooLarge,
    CoverUnavailable,
    InvalidCategory,
    InvalidYear,
    TextTooLong,
    InvalidSeriesDescription,
    ContentNotFound,
    InvalidContentOrder,
    InvalidProgress,
    InvalidRating,
    InvalidWatchDates,
    ProgressResetRequired,
    RewatchRequiresCompletion,
    WatchSessionChanged,
    InvalidPreferences,
    DuplicateTerm,
    TermNotFound,
    CategoryInUse,
    InvalidReplacementCategory,
}

impl From<SqlError> for CatalogError {
    fn from(error: SqlError) -> Self {
        match error {
            SqlError::QueryReturnedNoRows => Self::SeriesNotFound,
            _ => Self::CatalogUnavailable,
        }
    }
}

impl From<CoverError> for CatalogError {
    fn from(error: CoverError) -> Self {
        match error {
            CoverError::Invalid => Self::InvalidCover,
            CoverError::TooLarge => Self::CoverTooLarge,
            CoverError::Unavailable => Self::CoverUnavailable,
        }
    }
}

type Result<T> = std::result::Result<T, CatalogError>;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSeries {
    pub id: Option<i64>,
    pub title: String,
    pub genres: Vec<String>,
    pub description: String,
    pub cover: CoverChange,
    pub panorama: CoverChange,
    pub allow_duplicate: bool,
}

pub(super) fn open(data_dir: &Path) -> Result<rusqlite::Connection> {
    database::open(data_dir).map_err(|_| CatalogError::CatalogUnavailable)
}

pub fn list(data_dir: &Path) -> Result<Catalog> {
    let mut connection = open(data_dir)?;
    let transaction = connection.transaction()?;
    let catalog = repo::catalog(&transaction)?;
    transaction.commit()?;
    Ok(catalog)
}

pub fn save(data_dir: &Path, input: SaveSeries) -> Result<Catalog> {
    let title = input.title.trim();
    if !valid_label(title, 200) {
        return Err(CatalogError::InvalidTitle);
    }
    if input.description.chars().count() > 10_000 || input.description.contains('\0') {
        return Err(CatalogError::InvalidSeriesDescription);
    }
    if input.genres.len() > 50 {
        return Err(CatalogError::TooManyGenres);
    }
    let mut genres = BTreeMap::new();
    for name in &input.genres {
        let name = name.trim();
        if !valid_label(name, 60) {
            return Err(CatalogError::InvalidGenre);
        }
        genres
            .entry(normalize(name))
            .or_insert_with(|| name.to_owned());
    }
    let normalized = normalize(title);
    let cover = input.cover.prepare()?;
    let panorama = input.panorama.prepare_panorama()?;

    let mut connection = open(data_dir)?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let previous = input
        .id
        .map(|id| repo::series_keys(&transaction, id))
        .transpose()?;
    if !input.allow_duplicate
        && previous
            .as_ref()
            .is_none_or(|keys| keys.normalized_title != normalized)
        && repo::duplicate(&transaction, &normalized, input.id)?
    {
        return Err(CatalogError::DuplicateTitle);
    }
    let old_cover = previous.as_ref().and_then(|keys| keys.cover.as_deref());
    let old_panorama = previous.as_ref().and_then(|keys| keys.panorama.as_deref());
    cover.persist(data_dir, old_cover, |cover| {
        panorama.persist(data_dir, old_panorama, |panorama| {
            let id = repo::save_series(
                &transaction,
                input.id,
                SeriesValues {
                    title,
                    normalized: &normalized,
                    description: input.description.trim(),
                    cover,
                    panorama,
                },
            )?;
            repo::set_genres(&transaction, id, &genres)?;
            let catalog = repo::catalog(&transaction)?;
            transaction.commit()?;
            Ok(catalog)
        })
    })
}

pub fn favorite(data_dir: &Path, id: i64, favorite: bool) -> Result<Catalog> {
    let mut connection = open(data_dir)?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    if repo::set_favorite(&transaction, id, favorite)? == 0 {
        return Err(CatalogError::SeriesNotFound);
    }
    let catalog = repo::catalog(&transaction)?;
    transaction.commit()?;
    Ok(catalog)
}

pub fn delete(data_dir: &Path, id: i64) -> Result<Catalog> {
    let mut connection = open(data_dir)?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let keys = repo::series_keys(&transaction, id)?;
    let child_covers = repo::content::cover_keys(&transaction, id)?;
    repo::delete_series(&transaction, id)?;
    let catalog = repo::catalog(&transaction)?;
    transaction.commit()?;
    for key in child_covers
        .into_iter()
        .chain(keys.cover)
        .chain(keys.panorama)
    {
        covers::remove(data_dir, &key);
    }
    Ok(catalog)
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{Engine, engine::general_purpose::STANDARD};
    use image::{DynamicImage, ImageFormat};
    use std::{fs, io::Cursor};
    use tempfile::tempdir;

    fn draft(title: &str) -> SaveSeries {
        SaveSeries {
            id: None,
            title: title.into(),
            genres: Vec::new(),
            description: String::new(),
            cover: CoverChange::Keep,
            panorama: CoverChange::Keep,
            allow_duplicate: false,
        }
    }

    fn cover() -> Vec<u8> {
        let mut bytes = Cursor::new(Vec::new());
        DynamicImage::new_rgb8(4, 6)
            .write_to(&mut bytes, ImageFormat::Png)
            .unwrap();
        bytes.into_inner()
    }

    #[test]
    fn saves_reuses_updates_and_deletes_with_normalized_duplicate_confirmation() {
        let temp = tempdir().unwrap();
        let root = temp.path();
        let mut first = draft("Mi  Ánime");
        first.genres = vec!["Ciencia ficción".into(), "Drama".into()];
        let first = save(root, first).unwrap();
        let id = first.series[0].id;
        assert_eq!(first.series[0].genre_ids.len(), 2);

        let mut second = draft("MI ANIME");
        second.genres = vec!["ciencia ficcion".into(), "CIENCIA FICCIÓN".into()];
        assert_eq!(
            save(root, draft("mi anime")).unwrap_err(),
            CatalogError::DuplicateTitle
        );
        assert_eq!(list(root).unwrap().series.len(), 1);
        second.allow_duplicate = true;
        let catalog = save(root, second).unwrap();
        let other = catalog.series[1].id;
        assert_ne!(id, other);
        assert_eq!(catalog.genres.len(), 2);
        assert_eq!(catalog.series[1].genre_ids, [catalog.genres[0].id]);

        favorite(root, id, true).unwrap();
        let mut edit = draft("Otro título");
        edit.id = Some(id);
        edit.genres = vec!["Drama".into()];
        save(root, edit).unwrap();
        let reopened = list(root).unwrap();
        assert_eq!(reopened.series[1].title, "Otro título");
        assert!(reopened.series[1].favorite);
        assert_eq!(reopened.series[1].genre_ids, [reopened.genres[1].id]);
        assert!(!favorite(root, id, false).unwrap().series[1].favorite);
        assert_eq!(delete(root, id).unwrap().genres.len(), 2);
        assert_eq!(delete(root, id).unwrap_err(), CatalogError::SeriesNotFound);
    }

    #[test]
    fn copies_replaces_and_removes_only_managed_covers() {
        let temp = tempdir().unwrap();
        let root = temp.path().join("data");
        let external = temp.path().join("original.png");
        let bytes = cover();
        fs::write(&external, &bytes).unwrap();
        let replacement = || CoverChange::Replace {
            data: STANDARD.encode(fs::read(&external).unwrap()),
        };
        let mut input = draft("Serie ilustrada");
        input.cover = replacement();
        save(&root, input).unwrap();
        let catalog = list(&root).unwrap();
        let series = &catalog.series[0];
        let key = series.cover_key.as_deref().unwrap();
        assert_eq!(
            fs::read(root.join("covers").join(key).join("original")).unwrap(),
            bytes
        );
        assert!(covers::thumbnail(&root, key).is_some());
        assert!(covers::thumbnail(&root, "../original.png").is_none());

        let mut edit = draft(&series.title);
        edit.id = Some(series.id);
        edit.cover = replacement();
        let updated = save(&root, edit).unwrap();
        let next = updated.series[0].cover_key.as_deref().unwrap();
        assert_ne!(next, key);
        assert!(covers::thumbnail(&root, key).is_none());
        assert!(covers::thumbnail(&root, next).is_some());
        let mut edit = draft(&series.title);
        edit.id = Some(series.id);
        edit.cover = CoverChange::Remove;
        assert!(save(&root, edit).unwrap().series[0].cover_key.is_none());
        assert!(covers::thumbnail(&root, next).is_none());
        assert_eq!(fs::read(&external).unwrap(), bytes);
    }

    #[test]
    fn manages_panorama_independently_and_preserves_both_assets_on_failure() {
        let temp = tempdir().unwrap();
        let root = temp.path().join("data");
        let external = temp.path().join("panorama.png");
        let mut bytes = Cursor::new(Vec::new());
        DynamicImage::new_rgb8(1600, 900)
            .write_to(&mut bytes, ImageFormat::Png)
            .unwrap();
        fs::write(&external, bytes.get_ref()).unwrap();
        let replacement = || CoverChange::Replace {
            data: STANDARD.encode(fs::read(&external).unwrap()),
        };
        let mut input = draft("Serie");
        input.description = "Primera línea\nSegunda línea".into();
        input.cover = CoverChange::Replace {
            data: STANDARD.encode(cover()),
        };
        input.panorama = replacement();
        save(&root, input).unwrap();
        let reopened = list(&root).unwrap();
        let series = &reopened.series[0];
        let cover_key = series.cover_key.as_deref().unwrap();
        let first_panorama = series.panorama_key.as_deref().unwrap();
        assert_eq!(series.description, "Primera línea\nSegunda línea");
        let preview = covers::thumbnail(&root, first_panorama).unwrap();
        let image = image::load_from_memory(&preview).unwrap();
        assert_eq!((image.width(), image.height()), (1280, 720));

        let connection = database::open(&root).unwrap();
        connection.execute_batch(
            "CREATE TRIGGER fail_update BEFORE UPDATE ON series BEGIN SELECT RAISE(ABORT, 'test'); END;",
        ).unwrap();
        let mut edit = draft("Serie");
        edit.id = Some(series.id);
        edit.panorama = replacement();
        edit.cover = replacement();
        assert_eq!(
            save(&root, edit).unwrap_err(),
            CatalogError::CatalogUnavailable
        );
        assert_eq!(fs::read_dir(root.join("covers")).unwrap().count(), 2);
        assert!(covers::thumbnail(&root, cover_key).is_some());
        assert!(covers::thumbnail(&root, first_panorama).is_some());
        connection
            .execute_batch("DROP TRIGGER fail_update;")
            .unwrap();

        let mut edit = draft("Serie");
        edit.id = Some(series.id);
        edit.panorama = replacement();
        let updated = save(&root, edit).unwrap();
        let next_panorama = updated.series[0].panorama_key.as_deref().unwrap();
        assert_ne!(first_panorama, next_panorama);
        assert!(covers::thumbnail(&root, first_panorama).is_none());
        assert_eq!(updated.series[0].cover_key.as_deref(), Some(cover_key));

        let mut edit = draft("Serie");
        edit.id = Some(series.id);
        edit.panorama = CoverChange::Remove;
        assert!(save(&root, edit).unwrap().series[0].panorama_key.is_none());
        assert!(covers::thumbnail(&root, next_panorama).is_none());
        let mut edit = draft("Serie");
        edit.id = Some(series.id);
        edit.panorama = replacement();
        save(&root, edit).unwrap();
        delete(&root, series.id).unwrap();
        assert_eq!(fs::read_dir(root.join("covers")).unwrap().count(), 0);
        assert_eq!(fs::read(external).unwrap(), bytes.into_inner());
    }

    #[test]
    fn invalid_inputs_and_failed_writes_preserve_catalog_and_discard_new_cover() {
        let temp = tempdir().unwrap();
        let root = temp.path();
        assert_eq!(
            save(root, draft(" ")).unwrap_err(),
            CatalogError::InvalidTitle
        );
        let mut input = draft("Serie");
        input.description = "a".repeat(10_001);
        assert_eq!(
            save(root, input).unwrap_err(),
            CatalogError::InvalidSeriesDescription
        );
        let mut input = draft("Serie");
        input.cover = CoverChange::Replace {
            data: STANDARD.encode("not an image"),
        };
        assert_eq!(save(root, input).unwrap_err(), CatalogError::InvalidCover);
        let connection = database::open(root).unwrap();
        connection.execute_batch(
            "CREATE TRIGGER fail_genre BEFORE INSERT ON genres BEGIN SELECT RAISE(ABORT, 'test'); END;"
        ).unwrap();
        let mut input = draft("Serie");
        input.genres.push("Drama".into());
        input.cover = CoverChange::Replace {
            data: STANDARD.encode(cover()),
        };
        assert_eq!(
            save(root, input).unwrap_err(),
            CatalogError::CatalogUnavailable
        );
        assert!(list(root).unwrap().series.is_empty());
        assert_eq!(fs::read_dir(root.join("covers")).unwrap().count(), 0);
    }
}
