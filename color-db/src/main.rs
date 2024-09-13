use std::path::Path;

use sqlx::{sqlite::SqliteConnectOptions, Acquire, SqliteConnection, SqlitePool};

#[derive(Debug, Default)]
struct LDrawColor {
    code: u32,
    name: String,
    value: u32,
    edge: u32,
    alpha: Option<u8>,
    luminance: Option<u8>,
    finish: Option<String>,
}

impl LDrawColor {
    fn parse(line: &str) -> Option<(Self, Option<LDrawMaterial>)> {
        let mut color = LDrawColor::default();
        let mut words = line.split_whitespace();

        let Some("0") = words.next() else {
            return None;
        };
        let Some("!COLOUR") = words.next() else {
            return None;
        };

        color.name = words.next()?.to_owned();

        while let Some(key) = words.next() {
            match key {
                "CODE" => color.code = words.next()?.parse().ok()?,
                "VALUE" => color.value = words.next().and_then(parse_hex)?,
                "EDGE" => color.edge = words.next().and_then(parse_hex)?,
                "ALPHA" => color.alpha = Some(words.next()?.parse().ok()?),
                "LUMINANCE" => color.luminance = Some(words.next()?.parse().ok()?),
                "MATERIAL" => {
                    let material = LDrawMaterial::parse(words)?;
                    return Some((color, Some(material)));
                }
                _ => color.finish = Some(key.to_owned()),
            }
        }

        Some((color, None))
    }
}

#[derive(Debug, Default)]
struct LDrawMaterial {
    kind: String,
    value: u32,
    fraction: f32,
    volume_fraction: Option<f32>,
    size: Option<u32>,
    min_size: Option<u32>,
    max_size: Option<u32>,
}

impl LDrawMaterial {
    fn parse<'a>(mut words: impl Iterator<Item = &'a str>) -> Option<Self> {
        let mut material = Self::default();
        material.kind = words.next()?.to_owned();
        while let Some(key) = words.next() {
            match key {
                "VALUE" => material.value = words.next().and_then(parse_hex)?,
                "FRACTION" => material.fraction = words.next()?.parse().ok()?,
                "VFRACTION" => material.volume_fraction = Some(words.next()?.parse().ok()?),
                "SIZE" => material.size = Some(words.next()?.parse().ok()?),
                "MINSIZE" => material.min_size = Some(words.next()?.parse().ok()?),
                "MAXSIZE" => material.max_size = Some(words.next()?.parse().ok()?),
                _ => return None,
            }
        }
        Some(material)
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("colors.sqlite");
    std::fs::remove_file(&path)?;

    let options = SqliteConnectOptions::new()
        .filename(&path)
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;
    let mut pool_conn = pool.acquire().await?;
    let conn = pool_conn.acquire().await?;

    sqlx::migrate!("db/migrations").run(&mut *conn).await?;

    let x = include_str!("/mnt/c/program files/studio 2.0 earlyaccess/ldraw/ldconfig.ldr");
    insert_file(x, "studio earlyaccess", &mut *conn).await?;

    Ok(())
}

async fn insert_file(
    contents: &str,
    name: &str,
    conn: &mut SqliteConnection,
) -> Result<(), Box<dyn std::error::Error>> {
    let db_id = sqlx::query_scalar!(
        "INSERT INTO ldraw_database (id, name) VALUES (NULL, ?) RETURNING id",
        name
    )
    .fetch_one(&mut *conn)
    .await?;

    for line in contents.lines() {
        let Some((color, material)) = LDrawColor::parse(line) else {
            if line.contains("!COLO") {
                println!("WARNING: failed parse!");
            }
            continue;
        };

        sqlx::query!(
            "INSERT INTO ldraw_color
                (db, code, name, value, edge, alpha, luminance, finish)
            VALUES
                (?, ?, ?, ?, ?, ?, ?, ?)
            ",
            db_id,
            color.code,
            color.name,
            color.value,
            color.edge,
            color.alpha,
            color.luminance,
            color.finish,
        )
        .execute(&mut *conn)
        .await?;

        if let Some(material) = material {
            sqlx::query!(
                "INSERT INTO ldraw_secondary_material
                    (db, code, kind, value, fraction, volume_fraction, size, min_size, max_size)
                VALUES
                    (?, ?, ?, ?, ?, ?, ?, ?, ?)
                ",
                db_id,
                color.code,
                material.kind,
                material.value,
                material.fraction,
                material.volume_fraction,
                material.size,
                material.min_size,
                material.max_size,
            )
            .execute(&mut *conn)
            .await?;
        }
    }

    Ok(())
}

fn parse_hex(s: &str) -> Option<u32> {
    let s = s.strip_prefix("#")?;
    if s.len() != 6 {
        return None;
    }
    u32::from_str_radix(s, 0x10).ok()
}
