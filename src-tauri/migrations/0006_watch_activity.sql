ALTER TABLE watch_sessions ADD COLUMN activity_at TEXT;

UPDATE watch_sessions SET activity_at = (
    SELECT updated_at FROM contents WHERE contents.id = watch_sessions.content_id
) WHERE active = 1;

CREATE INDEX watch_sessions_continue ON watch_sessions(activity_at DESC, id DESC)
WHERE active = 1 AND status = 'watching';

CREATE TRIGGER watch_sessions_record_activity
AFTER UPDATE OF watched_episodes, status ON watch_sessions
WHEN NEW.watched_episodes <> OLD.watched_episodes OR NEW.status <> OLD.status
BEGIN
    UPDATE watch_sessions SET activity_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    WHERE id = NEW.id;
END;
