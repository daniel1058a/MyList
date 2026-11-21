use std::{collections::BTreeMap, io::Write, path::Path};

use serde::Deserialize;
use tempfile::NamedTempFile;

use crate::{domain::watch::Status, infrastructure::database, repositories};

use super::{
    backups::{BackupError, Result, verify_destination},
    preferences::{self, Language},
};

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Format {
    Csv,
    Markdown,
}

impl Format {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Markdown => "md",
        }
    }
}

struct Item {
    title: String,
    category: String,
    description: String,
    opinion: String,
    year: Option<i64>,
    rating: Option<i64>,
    status: String,
    watched: i64,
    total: Option<i64>,
}

fn csv(value: &str) -> String {
    let formula = value.trim_start().starts_with(['=', '+', '-', '@'])
        || value.starts_with(['\t', '\r', '\n']);
    format!(
        "\"{}{}\"",
        if formula { "'" } else { "" },
        value.replace('"', "\"\"")
    )
}

fn csv_row(output: &mut impl Write, fields: &[&str]) -> std::io::Result<()> {
    write!(
        output,
        "{}\r\n",
        fields
            .iter()
            .map(|field| csv(field))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn markdown(value: &str) -> String {
    let mut result = String::new();
    for character in value.chars() {
        match character {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '\r' => {}
            other => {
                if other.is_ascii_punctuation() {
                    result.push('\\');
                }
                result.push(other);
            }
        }
    }
    result
}

fn heading(value: &str) -> String {
    markdown(&value.replace(['\r', '\n'], " "))
}

fn labels(language: Language) -> [&'static str; 15] {
    match language {
        Language::Es => [
            "Serie",
            "Géneros",
            "Favorita",
            "Descripción general",
            "Promedio de la serie",
            "Contenido",
            "Categoría",
            "Año",
            "Calificación",
            "Estado",
            "Episodios vistos",
            "Episodios totales",
            "Descripción",
            "Opinión personal",
            "Biblioteca de MyList",
        ],
        Language::En => [
            "Series",
            "Genres",
            "Favorite",
            "Series description",
            "Series average",
            "Entry",
            "Category",
            "Year",
            "Rating",
            "Status",
            "Episodes watched",
            "Total episodes",
            "Description",
            "Personal opinion",
            "MyList library",
        ],
    }
}

fn status(value: &str, language: Language) -> Result<&'static str> {
    match (Status::parse(value), language) {
        (Some(Status::ToWatch), Language::Es) => Ok("Por ver"),
        (Some(Status::Watching), Language::Es) => Ok("Mirando"),
        (Some(Status::Finished), Language::Es) => Ok("Finalizado"),
        (Some(Status::ToWatch), Language::En) => Ok("To watch"),
        (Some(Status::Watching), Language::En) => Ok("Watching"),
        (Some(Status::Finished), Language::En) => Ok("Finished"),
        (None, _) => Err(BackupError::InvalidDatabase),
    }
}

fn optional(value: Option<i64>) -> String {
    value.map(|value| value.to_string()).unwrap_or_default()
}

fn decimal(value: String, language: Language) -> String {
    if language == Language::Es {
        value.replace('.', ",")
    } else {
        value
    }
}

pub fn export(root: &Path, target: &Path, format: Format) -> Result<()> {
    verify_destination(target, format.extension())?;
    let language = preferences::access(root, None)
        .map_err(|_| BackupError::InvalidDatabase)?
        .language;
    let mut connection = database::open(root)?;
    let tx = connection.transaction()?;
    let catalog = repositories::catalog(&tx)?;
    let mut items = BTreeMap::<i64, Vec<Item>>::new();
    {
        let mut statement = tx.prepare(
            "SELECT c.series_id,c.title,a.name,c.description,c.opinion,c.release_year,c.rating_tenths,
                w.status,w.watched_episodes,c.total_episodes
             FROM contents c JOIN categories a ON a.id=c.category_id
             JOIN watch_sessions w ON w.content_id=c.id AND w.active=1
             ORDER BY c.series_id,c.sort_order,c.id",
        )?;
        let mut rows = statement.query([])?;
        while let Some(row) = rows.next()? {
            items.entry(row.get(0)?).or_default().push(Item {
                title: row.get(1)?,
                category: row.get(2)?,
                description: row.get(3)?,
                opinion: row.get(4)?,
                year: row.get(5)?,
                rating: row.get(6)?,
                status: row.get(7)?,
                watched: row.get(8)?,
                total: row.get(9)?,
            });
        }
    }
    tx.commit()?;
    let mut stage = NamedTempFile::new_in(target.parent().ok_or(BackupError::Io)?)?;
    let output = stage.as_file_mut();
    let names = labels(language);
    let genres: BTreeMap<_, _> = catalog
        .genres
        .iter()
        .map(|genre| (genre.id, genre.name.as_str()))
        .collect();
    let csv_format = matches!(format, Format::Csv);
    if csv_format {
        output.write_all(&[0xef, 0xbb, 0xbf])?;
        csv_row(output, &names[..14])?;
    } else {
        writeln!(output, "# {}\n", names[14])?;
    }
    for series in catalog.series {
        let genres = series
            .genre_ids
            .iter()
            .filter_map(|id| genres.get(id).copied())
            .collect::<Vec<_>>()
            .join(" | ");
        let favorite = match (series.favorite, language) {
            (true, Language::Es) => "Sí",
            (true, Language::En) => "Yes",
            _ => "No",
        };
        let average = series
            .rating_average
            .map(|value| decimal(format!("{value:.2}"), language))
            .unwrap_or_default();
        let prefix = [
            series.title.as_str(),
            &genres,
            favorite,
            series.description.as_str(),
            &average,
        ];
        let children = items.remove(&series.id).unwrap_or_default();
        if !csv_format {
            writeln!(output, "## {}\n", heading(&series.title))?;
            for (index, value) in [(1, genres.as_str()), (2, favorite), (4, &average)] {
                if !value.is_empty() {
                    writeln!(output, "- **{}:** {}", names[index], heading(value))?;
                }
            }
            if !series.description.is_empty() {
                writeln!(output, "\n{}\n", markdown(&series.description))?;
            }
        } else if children.is_empty() {
            let mut fields = prefix.to_vec();
            fields.resize(14, "");
            csv_row(output, &fields)?;
        }
        for item in children {
            let state = status(&item.status, language)?;
            let year = optional(item.year);
            let rating = item
                .rating
                .map(|value| decimal(format!("{}.{}", value / 10, value % 10), language))
                .unwrap_or_default();
            let watched = item.watched.to_string();
            let total = optional(item.total);
            if csv_format {
                let mut fields = prefix.to_vec();
                fields.extend([
                    item.title.as_str(),
                    &item.category,
                    &year,
                    &rating,
                    state,
                    &watched,
                    &total,
                    &item.description,
                    &item.opinion,
                ]);
                csv_row(output, &fields)?;
            } else {
                writeln!(output, "\n### {}\n", heading(&item.title))?;
                for (index, value) in [
                    (6, item.category.as_str()),
                    (7, &year),
                    (8, &rating),
                    (9, state),
                    (10, &watched),
                    (11, &total),
                ] {
                    if !value.is_empty() {
                        writeln!(output, "- **{}:** {}", names[index], heading(value))?;
                    }
                }
                for (index, value) in [(12, &item.description), (13, &item.opinion)] {
                    if !value.is_empty() {
                        writeln!(output, "\n**{}**\n\n{}", names[index], markdown(value))?;
                    }
                }
                writeln!(output)?;
            }
        }
        if !csv_format {
            writeln!(output)?;
        }
    }
    output.sync_all()?;
    stage.persist(target).map_err(|_| BackupError::Io)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn exports_csv_for_excel_and_markdown_hierarchy_without_managed_paths() {
        let temporary = tempdir().unwrap();
        let root = temporary.path();
        let connection = database::open(root).unwrap();
        connection.execute_batch(
            r#"INSERT INTO series(title, normalized_title, favorite, description)
               VALUES ('Órbita, "Norte"','orbita',1,'Vista <personal>'), ('Sin capítulos','sin',0,'');
               INSERT INTO genres(name,normalized_name) VALUES ('Ciencia y ficción','ciencia');
               INSERT INTO series_genres VALUES (1,1);
               INSERT INTO categories(name,normalized_name) VALUES ('Temporada','temporada');
               INSERT INTO contents(series_id,category_id,title,sort_order,rating_tenths,total_episodes,description,opinion,release_year)
               VALUES (1,1,'[Piloto]',0,95,5,'Uno' || char(10) || '# Dos','=1+1',2025);
               UPDATE watch_sessions SET status='watching',watched_episodes=2;
               INSERT INTO settings VALUES ('catalog.preferences.v1','{"language":"en"}');"#,
        ).unwrap();
        let csv_path = root.join("export.csv");
        let md_path = root.join("export.md");
        export(root, &csv_path, Format::Csv).unwrap();
        export(root, &md_path, Format::Markdown).unwrap();
        let csv = fs::read_to_string(csv_path).unwrap();
        assert!(csv.starts_with("\u{feff}\"Series\",\"Genres\""));
        assert!(csv.contains("\"Órbita, \"\"Norte\"\"\""));
        assert!(csv.contains("\"9.5\",\"Watching\",\"2\",\"5\",\"Uno\n# Dos\",\"'=1+1\"\r\n"));
        assert!(csv.contains("\"Sin capítulos\""));
        let md = fs::read_to_string(md_path).unwrap();
        assert!(md.starts_with("# MyList library\n"));
        assert!(md.contains("### \\[Piloto\\]"));
        assert!(md.contains("**Status:** Watching"));
        assert!(md.contains("Vista &lt;personal&gt;"));
        assert!(md.contains("Uno\n\\# Dos"));
        assert!(!md.contains("covers/") && !csv.contains("database.sqlite3"));
    }
}
