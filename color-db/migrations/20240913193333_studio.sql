CREATE TABLE studio_database (
    id INTEGER NOT NULL
        PRIMARY KEY ASC,
    name TEXT NOT NULL
) STRICT;

CREATE TABLE studio_color (
    db INTEGER NOT NULL
        REFERENCES studio_database (id),

    studio_code INTEGER NOT NULL,
    bricklink_code INTEGER NULL,
    ldraw_code INTEGER NOT NULL,
    ldd_code INTEGER NULL,

    studio_name TEXT NOT NULL CHECK (studio_name IS NOT ''),
    bricklink_name TEXT NULL CHECK (bricklink_name IS NOT ''),
    ldraw_name TEXT NOT NULL CHECK (ldraw_name IS NOT ''),
    ldd_name TEXT NULL CHECK (ldraw_name IS NOT ''),
    
    rgb INTEGER NOT NULL CHECK (rgb BETWEEN 0 AND 0xFFFFFF),
    alpha REAL NOT NULL CHECK (alpha BETWEEN 0.0 AND 1.0),

    category_name TEXT NOT NULL,
    color_group_index INTEGER NOT NULL,
    note TEXT NOT NULL,
    instruction_rgb INTEGER NULL CHECK (rgb BETWEEN 0 and 0xFFFFFF),
    instruction_cmyk INTEGER NULL,

    UNIQUE (db, studio_code, ldd_code)
) STRICT;
