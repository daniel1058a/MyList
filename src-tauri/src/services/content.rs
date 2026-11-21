use std::path::Path;

use rusqlite::{Error as SqlError, Transaction, TransactionBehavior};
use serde::Deserialize;

use crate::{
    domain::{normalize, valid_label},
    infrastructure::covers::{self, CoverChange},
    repositories::{
        self,
        content::{self as repo, ContentValues, SeriesContents},
    },
};

use super::catalog::{CatalogError, open};
type Result<T> = std::result::Result<T, CatalogError>;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveContent {
    id: Option<i64>,
    series_id: i64,
    title: String,
    category: String,
    description: String,
    opinion: String,
    release_year: Option<i32>,
    cover: CoverChange,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Placement {
    Before,
    After,
}

pub(super) fn finish(transaction: Transaction<'_>, series_id: i64) -> Result<SeriesContents> {
    let contents = repo::list(&transaction, series_id)?;
    transaction.commit()?;
    Ok(contents)
}

fn existing_cover(
    transaction: &Transaction<'_>,
    series_id: i64,
    id: i64,
) -> Result<Option<String>> {
    repo::cover_key(transaction, series_id, id).map_err(|error| match error {
        SqlError::QueryReturnedNoRows => CatalogError::ContentNotFound,
        _ => error.into(),
    })
}

pub fn list(data_dir: &Path, series_id: i64) -> Result<SeriesContents> {
    let mut connection = open(data_dir)?;
    let transaction = connection.transaction()?;
    repositories::series_keys(&transaction, series_id)?;
    finish(transaction, series_id)
}

pub fn save(data_dir: &Path, input: SaveContent) -> Result<SeriesContents> {
    let title = input.title.trim();
    let category = input.category.trim();
    if !valid_label(title, 200) {
        return Err(CatalogError::InvalidTitle);
    }
    if !valid_label(category, 60) {
        return Err(CatalogError::InvalidCategory);
    }
    if input
        .release_year
        .is_some_and(|year| !(1..=9999).contains(&year))
    {
        return Err(CatalogError::InvalidYear);
    }
    if [&input.description, &input.opinion]
        .iter()
        .any(|text| text.chars().count() > 10_000 || text.contains('\0'))
    {
        return Err(CatalogError::TextTooLong);
    }
    let cover = input.cover.prepare()?;
    let mut connection = open(data_dir)?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    repositories::series_keys(&transaction, input.series_id)?;
    let old_cover = match input.id {
        Some(id) => existing_cover(&transaction, input.series_id, id)?,
        None => None,
    };
    cover.persist(data_dir, old_cover.as_deref(), |cover_key| {
        let category_id = repo::category(&transaction, category, &normalize(category))?;
        repo::save(
            &transaction,
            input.series_id,
            input.id,
            ContentValues {
                title,
                category_id,
                cover_key,
                description: &input.description,
                opinion: &input.opinion,
                release_year: input.release_year,
            },
        )?;
        repo::touch_series(&transaction, input.series_id)?;
        finish(transaction, input.series_id)
    })
}

pub fn delete(data_dir: &Path, series_id: i64, id: i64) -> Result<SeriesContents> {
    let mut connection = open(data_dir)?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let cover = existing_cover(&transaction, series_id, id)?;
    repo::delete(&transaction, series_id, id)?;
    repo::touch_series(&transaction, series_id)?;
    let contents = finish(transaction, series_id)?;
    if let Some(key) = cover {
        covers::remove(data_dir, &key);
    }
    Ok(contents)
}

pub fn reorder(
    data_dir: &Path,
    series_id: i64,
    id: i64,
    target_id: i64,
    placement: Placement,
) -> Result<SeriesContents> {
    let mut connection = open(data_dir)?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    repositories::series_keys(&transaction, series_id)?;
    let mut order = repo::order(&transaction, series_id)?;
    if move_relative(&mut order, id, target_id, placement)? {
        repo::write_order(&transaction, series_id, &order)?;
        repo::touch_series(&transaction, series_id)?;
    }
    finish(transaction, series_id)
}

fn move_relative(order: &mut [i64], id: i64, target_id: i64, placement: Placement) -> Result<bool> {
    let locate = |id| {
        order
            .iter()
            .position(|&item| item == id)
            .ok_or(CatalogError::InvalidContentOrder)
    };
    let from = locate(id)?;
    let target = locate(target_id)?;
    if from == target {
        return Ok(false);
    }
    let insertion = target + usize::from(matches!(placement, Placement::After));
    let to = insertion - usize::from(insertion > from);
    if from < to {
        order[from..=to].rotate_left(1);
    } else if to < from {
        order[to..=from].rotate_right(1);
    }
    Ok(from != to)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::catalog::{self, SaveSeries};
    use base64::{Engine, engine::general_purpose::STANDARD};
    use image::{DynamicImage, ImageFormat};
    use std::{fs, io::Cursor};
    use tempfile::tempdir;

    fn series(root: &Path, title: &str) -> i64 {
        catalog::save(
            root,
            SaveSeries {
                id: None,
                title: title.into(),
                genres: Vec::new(),
                description: String::new(),
                cover: CoverChange::Keep,
                panorama: CoverChange::Keep,
                allow_duplicate: false,
            },
        )
        .unwrap()
        .series
        .into_iter()
        .find(|series| series.title == title)
        .unwrap()
        .id
    }

    fn draft(series_id: i64, title: &str, category: &str) -> SaveContent {
        SaveContent {
            id: None,
            series_id,
            title: title.into(),
            category: category.into(),
            description: String::new(),
            opinion: String::new(),
            release_year: None,
            cover: CoverChange::Keep,
        }
    }

    #[test]
    fn moves_in_both_directions_without_changing_other_ids() {
        let mut ids = [1, 2, 3, 4];
        assert!(move_relative(&mut ids, 1, 3, Placement::After).unwrap());
        assert_eq!(ids, [2, 3, 1, 4]);
        assert!(move_relative(&mut ids, 4, 2, Placement::Before).unwrap());
        assert_eq!(ids, [4, 2, 3, 1]);
        assert!(!move_relative(&mut ids, 2, 3, Placement::Before).unwrap());
        assert_eq!(
            move_relative(&mut ids, 7, 3, Placement::After).unwrap_err(),
            CatalogError::InvalidContentOrder
        );
    }

    #[test]
    fn persists_multiline_fields_reuses_categories_and_scopes_reordering() {
        let temp = tempdir().unwrap();
        let root = temp.path();
        let a = series(root, "Serie A");
        let b = series(root, "Serie B");
        let first = save(root, draft(a, "Primero", "Capítulo especial"))
            .unwrap()
            .items[0]
            .id;
        let mut second = draft(a, "Segundo", "Corto");
        second.description = "Primera línea\nSegunda línea".into();
        second.opinion = "Opinión\nAparte".into();
        second.release_year = Some(2021);
        let second = save(root, second).unwrap().items[1].id;
        let other = save(root, draft(b, "Otro", "CAPITULO  ESPECIAL")).unwrap();
        let other_id = other.items[0].id;
        assert_eq!(other.categories.len(), 2);
        assert_eq!(
            other.items[0].category_id,
            list(root, a).unwrap().items[0].category_id
        );
        assert_eq!(
            reorder(root, a, second, other_id, Placement::Before).unwrap_err(),
            CatalogError::InvalidContentOrder
        );
        assert_eq!(
            delete(root, a, other_id).unwrap_err(),
            CatalogError::ContentNotFound
        );

        let ordered = reorder(root, a, second, first, Placement::Before).unwrap();
        assert_eq!(
            ordered.items.iter().map(|item| item.id).collect::<Vec<_>>(),
            [second, first]
        );
        let reopened = list(root, a).unwrap();
        assert_eq!(reopened.items[0].id, second);
        assert_eq!(
            reopened.items[0].description,
            "Primera línea\nSegunda línea"
        );
        assert_eq!(reopened.items[0].opinion, "Opinión\nAparte");
        assert_eq!(reopened.items[0].release_year, Some(2021));
        let mut invalid = draft(a, "Nuevo", "No debe crearse");
        invalid.release_year = Some(10000);
        assert_eq!(save(root, invalid).unwrap_err(), CatalogError::InvalidYear);
        assert_eq!(list(root, a).unwrap().categories.len(), 2);
    }

    #[test]
    fn removes_managed_covers_after_item_and_series_deletion() {
        let temp = tempdir().unwrap();
        let root = temp.path().join("data");
        let original = temp.path().join("original.png");
        let mut bytes = Cursor::new(Vec::new());
        DynamicImage::new_rgb8(4, 6)
            .write_to(&mut bytes, ImageFormat::Png)
            .unwrap();
        fs::write(&original, bytes.get_ref()).unwrap();
        let replacement = || CoverChange::Replace {
            data: STANDARD.encode(fs::read(&original).unwrap()),
        };
        let a = series(&root, "Serie A");
        let b = series(&root, "Serie B");
        let mut first = draft(a, "Uno", "Documental");
        first.cover = replacement();
        let first = save(&root, first).unwrap();
        let id = first.items[0].id;
        let old_key = first.items[0].cover_key.as_deref().unwrap();
        let mut updated = draft(a, "Edición", "Documental");
        updated.id = Some(id);
        updated.cover = replacement();
        let updated = save(&root, updated).unwrap();
        let key = updated.items[0].cover_key.as_deref().unwrap();
        assert_eq!(updated.items[0].title, "Edición");
        assert!(covers::thumbnail(&root, old_key).is_none());
        assert!(covers::thumbnail(&root, key).is_some());
        assert!(delete(&root, a, id).unwrap().items.is_empty());
        assert!(covers::thumbnail(&root, key).is_none());

        let mut child = draft(a, "Parte", "Documental");
        child.cover = replacement();
        let child = save(&root, child).unwrap();
        let child_key = child.items[0].cover_key.as_deref().unwrap();
        save(&root, draft(b, "Conservado", "Documental")).unwrap();
        catalog::delete(&root, a).unwrap();
        let connection = open(&root).unwrap();
        assert!(repo::order(&connection, a).unwrap().is_empty());
        assert_eq!(list(&root, b).unwrap().categories.len(), 1);
        assert_eq!(list(&root, b).unwrap().items.len(), 1);
        assert!(covers::thumbnail(&root, child_key).is_none());
        assert_eq!(fs::read(original).unwrap(), bytes.into_inner());
    }
}
