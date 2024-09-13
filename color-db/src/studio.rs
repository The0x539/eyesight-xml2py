use sqlx::SqliteConnection;

use crate::parse_hex;

#[derive(Default)]
pub struct StudioColor {
    pub studio_code: i32,
    pub bricklink_code: Option<i32>,
    pub ldraw_code: i32,
    pub ldd_code: Option<i32>,

    pub studio_name: String,
    pub bricklink_name: Option<String>,
    pub ldraw_name: String,
    pub ldd_name: Option<String>,

    pub rgb: u32,
    pub alpha: f32,

    pub category_name: String,
    pub color_group_index: i32,
    pub note: String,
    pub instruction_rgb: Option<u32>,
    pub instruction_cmyk: Option<u32>,
    pub category_nickname: Option<String>,
}

impl StudioColor {
    pub fn parse(line: &str) -> Option<Self> {
        let mut words = line.split('\t').map(|s| (s != "").then_some(s)).fuse();

        let mut color = StudioColor::default();

        color.studio_code = words.next()??.parse().ok()?;
        if let Some(w) = words.next()? {
            color.bricklink_code = Some(w.parse().ok()?);
        }
        color.ldraw_code = words.next()??.parse().ok()?;
        if let Some(w) = words.next()? {
            color.bricklink_code = Some(w.parse().ok()?);
        }

        color.studio_name = words.next()??.to_owned();
        color.bricklink_name = words.next()?.map(String::from);
        color.ldraw_name = words.next()??.to_owned();
        color.ldd_name = words.next()?.map(String::from);

        color.rgb = words.next()?.and_then(parse_hex)?;
        color.alpha = words.next()??.parse().ok()?;

        color.category_name = words.next()??.to_owned();
        color.color_group_index = words.next()??.parse().ok()?;
        color.note = words.next()??.to_owned();

        if let Some(Some(w)) = words.next() {
            color.instruction_rgb = Some(parse_hex(w)?);
        }

        if let Some(Some(w)) = words.next() {
            color.instruction_cmyk = Some(parse_cmyk(w)?);
        }

        if let Some(Some(w)) = words.next() {
            color.category_nickname = Some(w.to_owned());
        }

        if words.next().is_some() {
            return None;
        }

        Some(color)
    }
}

pub async fn insert_file(
    contents: &str,
    name: &str,
    conn: &mut SqliteConnection,
) -> Result<(), sqlx::Error> {
    let db_id = sqlx::query_scalar!(
        "INSERT INTO studio_database VALUES (NULL, ?) RETURNING id",
        name
    )
    .fetch_one(&mut *conn)
    .await?;

    let mut lines = contents.lines();

    let header = lines.next().unwrap();
    let mut columns = header.split('\t').collect::<Vec<_>>();
    if columns.len() == 16 {
        assert_eq!(columns.pop(), Some("Categogy NickName"));
    }
    assert_eq!(
        columns,
        [
            "Studio Color Code",
            "BL Color Code",
            "LDraw Color Code",
            "LDD color code",
            "Studio Color Name",
            "BL Color Name",
            "LDraw Color Name",
            "LDD Color Name",
            "RGB value",
            "Alpha",
            "CategoryName",
            "Color Group Index",
            "note",
            "Ins_RGB",
            "Ins_CMYK",
        ]
    );

    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let Some(color) = StudioColor::parse(line) else {
            println!("WARNING: studio line parse error");
            continue;
        };

        sqlx::query!(
            "INSERT INTO studio_color
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            db_id,
            color.studio_code,
            color.bricklink_code,
            color.ldraw_code,
            color.ldd_code,
            color.studio_name,
            color.bricklink_name,
            color.ldraw_name,
            color.ldd_name,
            color.rgb,
            color.alpha,
            color.category_name,
            color.color_group_index,
            color.note,
            color.instruction_rgb,
            color.instruction_cmyk,
        )
        .execute(&mut *conn)
        .await?;
    }

    Ok(())
}

fn parse_cmyk(s: &str) -> Option<u32> {
    let mut iter = s.splitn(4, ',').filter_map(|x| x.parse::<u8>().ok());
    let cmyk = [iter.next()?, iter.next()?, iter.next()?, iter.next()?];
    Some(u32::from_be_bytes(cmyk))
}

#[cfg(test)]
#[test]
fn test_parse_line() {
    let base =
        include_str!("/mnt/c/program files/studio 2.0 earlyaccess/data/StudioColorDefinition.txt");

    let custom =
        include_str!("/mnt/c/program files/studio 2.0 earlyaccess/data/CustomColorDefinition.txt");

    let lines = base.lines().skip(1).chain(custom.lines().skip(1));

    for line in lines {
        println!();
        assert!(StudioColor::parse(line).is_some(), "{line}");
    }
}
