use crate::parse_hex;
use sqlx::SqliteConnection;

#[derive(Debug, Default)]
pub struct LDrawColor {
    pub code: u32,
    pub name: String,
    pub value: u32,
    pub edge: u32,
    pub alpha: Option<u8>,
    pub luminance: Option<u8>,
    pub finish: Option<String>,
}

impl LDrawColor {
    pub fn parse(line: &str) -> Option<(Self, Option<LDrawMaterial>)> {
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
pub struct LDrawMaterial {
    pub kind: String,
    pub value: u32,
    pub fraction: f32,
    pub volume_fraction: Option<f32>,
    pub size: Option<u32>,
    pub min_size: Option<u32>,
    pub max_size: Option<u32>,
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

pub async fn insert_file(
    contents: &str,
    name: &str,
    conn: &mut SqliteConnection,
) -> Result<(), sqlx::Error> {
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
