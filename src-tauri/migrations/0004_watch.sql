ALTER TABLE contents ADD COLUMN total_episodes INTEGER
    CHECK (total_episodes BETWEEN 1 AND 4294967295);
ALTER TABLE contents ADD COLUMN rating_tenths INTEGER CHECK (rating_tenths BETWEEN 10 AND 100);

CREATE TABLE watch_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content_id INTEGER NOT NULL REFERENCES contents(id) ON DELETE CASCADE,
    watched_episodes INTEGER NOT NULL DEFAULT 0 CHECK (watched_episodes BETWEEN 0 AND 4294967295),
    status TEXT NOT NULL DEFAULT 'to_watch' CHECK (status IN ('to_watch', 'watching', 'finished')),
    started_on TEXT,
    finished_on TEXT,
    completed INTEGER NOT NULL DEFAULT 0 CHECK (completed IN (0, 1)),
    active INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
    CHECK (started_on IS NULL OR finished_on IS NULL OR started_on <= finished_on),
    CHECK (status <> 'to_watch' OR watched_episodes = 0),
    CHECK (status <> 'finished' OR completed = 1)
) STRICT;

CREATE INDEX watch_sessions_content ON watch_sessions(content_id);
CREATE UNIQUE INDEX watch_sessions_one_active ON watch_sessions(content_id) WHERE active = 1;

INSERT INTO watch_sessions (content_id) SELECT id FROM contents;

CREATE TRIGGER contents_start_watch_session AFTER INSERT ON contents
BEGIN
    INSERT INTO watch_sessions (content_id) VALUES (NEW.id);
END;
