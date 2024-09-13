CREATE TABLE eyesight_database (
    id INTEGER NOT NULL
        PRIMARY KEY ASC,
    name TEXT NOT NULL
) STRICT;

CREATE TABLE eyesight_color (
    db INTEGER NOT NULL
        REFERENCES eyesight_database (id),

    name TEXT NOT NULL,
    category TEXT NOT NULL,

    red REAL NOT NULL,
    green REAL NOT NULL,
    blue REAL NOT NULL,

    PRIMARY KEY (db, name)
) STRICT;
