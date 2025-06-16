CREATE TABLE IF NOT EXISTS NOTES
(
    id          BLOB        PRIMARY KEY,
    owner       BLOB        NOT NULL,
    content     TEXT        NOT NULL,
    checked     BOOLEAN     NOT NULL
);
