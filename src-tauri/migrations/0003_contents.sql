CREATE TABLE categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    normalized_name TEXT NOT NULL UNIQUE
) STRICT;

CREATE TABLE contents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    series_id INTEGER NOT NULL REFERENCES series(id) ON DELETE CASCADE,
    category_id INTEGER NOT NULL REFERENCES categories(id) ON DELETE RESTRICT,
    title TEXT NOT NULL,
    cover_key TEXT UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    opinion TEXT NOT NULL DEFAULT '',
    release_year INTEGER CHECK (release_year BETWEEN 1 AND 9999),
    sort_order INTEGER NOT NULL CHECK (sort_order >= 0),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) STRICT;

CREATE INDEX contents_series_order ON contents(series_id, sort_order, id);
CREATE INDEX contents_category ON contents(category_id);
