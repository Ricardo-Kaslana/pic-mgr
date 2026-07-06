pub const CREATE_TABLES_SQL: &str = "
CREATE TABLE IF NOT EXISTS workspaces (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT NOT NULL,
    path       TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS images (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id    INTEGER NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    relative_path   TEXT NOT NULL,
    file_name       TEXT NOT NULL,
    file_size       INTEGER,
    file_hash       TEXT,
    width           INTEGER,
    height          INTEGER,
    format          TEXT,
    thumbnail_path  TEXT,
    notes           TEXT DEFAULT '',
    rating          INTEGER DEFAULT 0 CHECK(rating >= 0 AND rating <= 5),
    created_at      TEXT,
    file_created_at TEXT,
    updated_at      TEXT NOT NULL DEFAULT (datetime('now')),
    indexed_at      TEXT NOT NULL DEFAULT (datetime('now')),
    last_viewed_at  TEXT,
    UNIQUE(workspace_id, relative_path)
);

CREATE TABLE IF NOT EXISTS tags (
    id    INTEGER PRIMARY KEY AUTOINCREMENT,
    name  TEXT NOT NULL UNIQUE,
    color TEXT
);

CREATE TABLE IF NOT EXISTS image_tags (
    image_id INTEGER NOT NULL REFERENCES images(id) ON DELETE CASCADE,
    tag_id   INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (image_id, tag_id)
);

CREATE VIRTUAL TABLE IF NOT EXISTS images_fts USING fts5(
    file_name,
    notes,
    content='images',
    content_rowid='id'
);

CREATE TRIGGER IF NOT EXISTS images_fts_insert AFTER INSERT ON images BEGIN
    INSERT INTO images_fts(rowid, file_name, notes) VALUES (new.id, new.file_name, new.notes);
END;

CREATE TRIGGER IF NOT EXISTS images_fts_delete AFTER DELETE ON images BEGIN
    INSERT INTO images_fts(images_fts, rowid) VALUES('delete', old.id);
END;

CREATE TRIGGER IF NOT EXISTS images_fts_update AFTER UPDATE ON images BEGIN
    INSERT INTO images_fts(images_fts, rowid) VALUES('delete', old.id);
    INSERT INTO images_fts(rowid, file_name, notes) VALUES (new.id, new.file_name, new.notes);
END;

CREATE INDEX IF NOT EXISTS idx_images_hash ON images(file_hash);
CREATE INDEX IF NOT EXISTS idx_image_tags_tag_id ON image_tags(tag_id);
CREATE INDEX IF NOT EXISTS idx_images_workspace_id ON images(workspace_id);
CREATE INDEX IF NOT EXISTS idx_images_workspace_indexed ON images(workspace_id, indexed_at DESC);
";
