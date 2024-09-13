CREATE TABLE ldraw_database (
    id INTEGER NOT NULL
        PRIMARY KEY ASC,
    name TEXT NOT NULL
) STRICT;

CREATE TABLE ldraw_color (
    db INTEGER NOT NULL
        REFERENCES ldraw_database (id),
    code INTEGER NOT NULL,
    name TEXT NOT NULL,
    value INTEGER NOT NULL
        CHECK (value BETWEEN 0 AND 0xFFFFFF),
    edge INTEGER NOT NULL
        CHECK (edge BETWEEN 0 AND 0xFFFFFF),
    alpha INTEGER NULL
        CHECK (alpha BETWEEN 0 AND 0xFF),
    luminance INTEGER NULL
        CHECK (luminance BETWEEN 0 AND 0xFF),
    finish TEXT NULL,

    PRIMARY KEY (db, code)
) STRICT;

CREATE TABLE ldraw_secondary_material (
    db INTEGER NOT NULL,
    code INTEGER NOT NULL,
    kind TEXT NOT NULL,
    value INTEGER NOT NULL
        CHECK (value BETWEEN 0 AND 0xFFFFFF),
    fraction REAL NOT NULL
        CHECK (fraction BETWEEN 0.0 AND 1.0),
    volume_fraction REAL NULL
        CHECK (volume_fraction BETWEEN 0.0 AND 1.0),
    size INTEGER NULL
        CHECK (size > 0),
    min_size REAL NULL
        CHECK (min_size > 0),
    max_size REAL NULL
        CHECK (max_size > min_size),

    CHECK ((min_size IS NULL) == (max_size IS NULL)),
    CHECK ((min_size IS NULL) != (size IS NULL)),
    PRIMARY KEY (db, code),
    FOREIGN KEY (db, code) REFERENCES ldraw_color (db, code)
) STRICT;
