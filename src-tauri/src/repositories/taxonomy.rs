use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Group {
    Genre,
    Category,
}

impl Group {
    fn table(self) -> &'static str {
        match self {
            Self::Genre => "genres",
            Self::Category => "categories",
        }
    }

    fn relation(self) -> &'static str {
        match self {
            Self::Genre => "series_genres WHERE genre_id",
            Self::Category => "contents WHERE category_id",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Term {
    pub id: i64,
    pub name: String,
    pub usage: i64,
}

pub fn list(connection: &Connection, group: Group) -> Result<Vec<Term>> {
    let sql = format!(
        "SELECT term.id, term.name, (SELECT COUNT(*) FROM {} = term.id)
         FROM {} term ORDER BY term.normalized_name, term.id",
        group.relation(),
        group.table(),
    );
    connection
        .prepare(&sql)?
        .query_map([], |row| {
            Ok(Term {
                id: row.get(0)?,
                name: row.get(1)?,
                usage: row.get(2)?,
            })
        })?
        .collect()
}

pub fn exists(connection: &Connection, group: Group, id: i64) -> Result<bool> {
    let sql = format!(
        "SELECT EXISTS (SELECT 1 FROM {} WHERE id = ?1)",
        group.table()
    );
    connection.query_row(&sql, [id], |row| row.get(0))
}

pub fn duplicate(
    connection: &Connection,
    group: Group,
    normalized: &str,
    id: Option<i64>,
) -> Result<bool> {
    let sql = format!(
        "SELECT EXISTS (SELECT 1 FROM {} WHERE normalized_name = ?1 AND id IS NOT ?2)",
        group.table(),
    );
    connection.query_row(&sql, params![normalized, id], |row| row.get(0))
}

pub fn save(
    connection: &Connection,
    group: Group,
    id: Option<i64>,
    name: &str,
    normalized: &str,
) -> Result<()> {
    if let Some(id) = id {
        let sql = format!(
            "UPDATE {} SET name = ?1, normalized_name = ?2 WHERE id = ?3",
            group.table()
        );
        connection.execute(&sql, params![name, normalized, id])?;
    } else {
        let sql = format!(
            "INSERT INTO {} (name, normalized_name) VALUES (?1, ?2)",
            group.table()
        );
        connection.execute(&sql, params![name, normalized])?;
    }
    Ok(())
}

pub fn usage(connection: &Connection, group: Group, id: i64) -> Result<i64> {
    let sql = format!("SELECT COUNT(*) FROM {} = ?1", group.relation());
    connection.query_row(&sql, [id], |row| row.get(0))
}

pub fn touch_linked_series(connection: &Connection, group: Group, id: i64) -> Result<()> {
    let sql = format!(
        "UPDATE series SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
         WHERE id IN (SELECT series_id FROM {} = ?1)",
        group.relation(),
    );
    connection.execute(&sql, [id])?;
    Ok(())
}

pub fn reassign(connection: &Connection, from: i64, to: i64) -> Result<()> {
    connection.execute(
        "UPDATE contents SET category_id = ?2, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')
         WHERE category_id = ?1",
        [from, to],
    )?;
    Ok(())
}

pub fn delete(connection: &Connection, group: Group, id: i64) -> Result<()> {
    if matches!(group, Group::Genre) {
        connection.execute("DELETE FROM series_genres WHERE genre_id = ?1", [id])?;
    }
    let sql = format!("DELETE FROM {} WHERE id = ?1", group.table());
    connection.execute(&sql, [id])?;
    Ok(())
}
