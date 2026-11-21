ALTER TABLE series ADD COLUMN description TEXT NOT NULL DEFAULT '';
ALTER TABLE series ADD COLUMN panorama_key TEXT;
CREATE UNIQUE INDEX series_panorama_key ON series(panorama_key) WHERE panorama_key IS NOT NULL;
