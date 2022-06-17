CREATE TABLE IF NOT EXISTS tiddlers 
(
    title TEXT UNIQUE PRIMARY KEY,
    revision INTEGER,
    meta BLOB
);
CREATE INDEX IF NOT EXISTS tiddlers_title_index ON tiddlers (title);