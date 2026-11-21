use std::path::Path;

use rusqlite::TransactionBehavior;
use serde::{Deserialize, Serialize};

use super::catalog::{CatalogError, open};
use crate::{
    domain::{normalize, valid_label},
    repositories::{
        self, Catalog,
        taxonomy::{self as repo, Group, Term},
    },
};

#[derive(Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum Change {
    Save {
        group: Group,
        id: Option<i64>,
        name: String,
    },
    Delete {
        group: Group,
        id: i64,
        replacement_id: Option<i64>,
    },
}

#[derive(Debug, Serialize)]
pub struct Taxonomy {
    pub genres: Vec<Term>,
    pub categories: Vec<Term>,
    pub catalog: Option<Catalog>,
}

pub fn access(data_dir: &Path, change: Option<Change>) -> Result<Taxonomy, CatalogError> {
    let mut connection = open(data_dir)?;
    let behavior = if change.is_some() {
        TransactionBehavior::Immediate
    } else {
        TransactionBehavior::Deferred
    };
    let transaction = connection.transaction_with_behavior(behavior)?;
    let catalog = if let Some(change) = change {
        apply(&transaction, change)?;
        Some(repositories::catalog(&transaction)?)
    } else {
        None
    };
    let result = Taxonomy {
        genres: repo::list(&transaction, Group::Genre)?,
        categories: repo::list(&transaction, Group::Category)?,
        catalog,
    };
    transaction.commit()?;
    Ok(result)
}

fn ensure_term(
    connection: &rusqlite::Connection,
    group: Group,
    id: i64,
) -> Result<(), CatalogError> {
    if !repo::exists(connection, group, id)? {
        return Err(CatalogError::TermNotFound);
    }
    Ok(())
}

fn apply(connection: &rusqlite::Connection, change: Change) -> Result<(), CatalogError> {
    match change {
        Change::Save { group, id, name } => {
            let name = name.trim();
            if !valid_label(name, 60) {
                return Err(match group {
                    Group::Genre => CatalogError::InvalidGenre,
                    Group::Category => CatalogError::InvalidCategory,
                });
            }
            let normalized = normalize(name);
            if repo::duplicate(connection, group, &normalized, id)? {
                return Err(CatalogError::DuplicateTerm);
            }
            if let Some(id) = id {
                ensure_term(connection, group, id)?;
                repo::touch_linked_series(connection, group, id)?;
            }
            repo::save(connection, group, id, name, &normalized)?;
        }
        Change::Delete {
            group,
            id,
            replacement_id,
        } => {
            ensure_term(connection, group, id)?;
            repo::touch_linked_series(connection, group, id)?;
            if matches!(group, Group::Category) {
                let usage = repo::usage(connection, group, id)?;
                if usage > 0 {
                    let to = replacement_id.ok_or(CatalogError::CategoryInUse)?;
                    if to == id || !repo::exists(connection, group, to)? {
                        return Err(CatalogError::InvalidReplacementCategory);
                    }
                    repo::reassign(connection, id, to)?;
                }
            }
            repo::delete(connection, group, id)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn seed(root: &Path) {
        open(root).unwrap().execute_batch(
            "INSERT INTO series (title, normalized_title) VALUES ('Serie', 'serie');
             INSERT INTO genres (name, normalized_name) VALUES ('Ciencia', 'ciencia'), ('Drama', 'drama');
             INSERT INTO series_genres VALUES (1, 1);
             INSERT INTO categories (name, normalized_name) VALUES ('Corto', 'corto'), ('Película', 'pelicula');
             INSERT INTO contents (series_id, category_id, title, sort_order, description, rating_tenths, cover_key)
             VALUES (1,1,'Contenido',0,'Conservar',85,'existing-key');",
        ).unwrap();
    }

    fn save(
        root: &Path,
        group: Group,
        id: Option<i64>,
        name: &str,
    ) -> Result<Taxonomy, CatalogError> {
        access(
            root,
            Some(Change::Save {
                group,
                id,
                name: name.into(),
            }),
        )
    }

    fn delete(
        root: &Path,
        group: Group,
        id: i64,
        replacement_id: Option<i64>,
    ) -> Result<Taxonomy, CatalogError> {
        access(
            root,
            Some(Change::Delete {
                group,
                id,
                replacement_id,
            }),
        )
    }

    #[test]
    fn creates_and_renames_terms_with_normalized_uniqueness_and_stable_links() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        seed(root);
        let created = save(root, Group::Genre, None, "  Acción  ").unwrap();
        assert!(
            created
                .genres
                .iter()
                .any(|term| term.name == "Acción" && term.usage == 0)
        );
        let created = save(root, Group::Category, None, "Extra").unwrap();
        assert!(
            created
                .categories
                .iter()
                .any(|term| term.name == "Extra" && term.usage == 0)
        );
        assert_eq!(
            save(root, Group::Category, None, "  ").unwrap_err(),
            CatalogError::InvalidCategory
        );
        assert_eq!(
            save(root, Group::Genre, Some(1), " DRÁMA ").unwrap_err(),
            CatalogError::DuplicateTerm
        );
        assert_eq!(
            save(root, Group::Category, None, "PELICULA").unwrap_err(),
            CatalogError::DuplicateTerm
        );
        let renamed = save(root, Group::Genre, Some(1), "Ciencia ficción").unwrap();
        assert!(
            renamed
                .genres
                .iter()
                .any(|term| term.id == 1 && term.usage == 1 && term.name == "Ciencia ficción")
        );
        assert_eq!(renamed.catalog.unwrap().series[0].genre_ids, [1]);
        let renamed = save(root, Group::Category, Some(1), "Cortometraje").unwrap();
        assert!(
            renamed
                .categories
                .iter()
                .any(|term| term.id == 1 && term.usage == 1 && term.name == "Cortometraje")
        );
        assert_eq!(
            super::super::content::list(root, 1).unwrap().items[0].category_id,
            1
        );
    }

    #[test]
    fn deletes_terms_without_deleting_related_data() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        seed(root);
        assert_eq!(
            delete(root, Group::Category, 1, None).unwrap_err(),
            CatalogError::CategoryInUse
        );
        for to in [1, 99] {
            assert_eq!(
                delete(root, Group::Category, 1, Some(to)).unwrap_err(),
                CatalogError::InvalidReplacementCategory
            );
        }
        let moved = delete(root, Group::Category, 1, Some(2)).unwrap();
        assert_eq!(moved.categories.len(), 1);
        assert_eq!(moved.categories[0].usage, 1);
        let catalog = delete(root, Group::Genre, 1, None)
            .unwrap()
            .catalog
            .unwrap();
        assert!(catalog.series[0].genre_ids.is_empty());
        let current = super::super::content::list(root, 1).unwrap();
        let item = &current.items[0];
        assert_eq!(item.category_id, 2);
        assert_eq!(item.description, "Conservar");
        assert_eq!(item.watch.rating_tenths, Some(85));
        assert_eq!(item.cover_key.as_deref(), Some("existing-key"));
        assert_eq!(
            super::super::watch::history(root, 1, item.id)
                .unwrap()
                .len(),
            1
        );
        let connection = open(root).unwrap();
        assert!(
            !connection
                .prepare("PRAGMA foreign_key_check")
                .unwrap()
                .exists([])
                .unwrap()
        );
    }

    #[test]
    fn rolls_back_reassignment_if_category_deletion_fails() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        seed(root);
        open(root).unwrap().execute_batch(
            "CREATE TRIGGER fail_delete BEFORE DELETE ON categories BEGIN SELECT RAISE(ABORT, 'test'); END;",
        ).unwrap();
        assert_eq!(
            delete(root, Group::Category, 1, Some(2)).unwrap_err(),
            CatalogError::CatalogUnavailable
        );
        assert_eq!(
            super::super::content::list(root, 1).unwrap().items[0].category_id,
            1
        );
        assert_eq!(access(root, None).unwrap().categories.len(), 2);
    }
}
