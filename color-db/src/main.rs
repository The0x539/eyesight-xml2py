use std::{collections::HashMap, path::Path};

use sqlx::{sqlite::SqliteConnectOptions, Acquire, SqliteConnection, SqlitePool};

pub mod ldraw;

pub mod studio;

pub mod eyesight;

type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("colors.sqlite");
    // let _ = std::fs::remove_file(&path);

    let options = SqliteConnectOptions::new()
        .filename(&path)
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;
    let mut pool_conn = pool.acquire().await?;
    let conn = pool_conn.acquire().await?;

    // sqlx::migrate!("./migrations").run(&mut *conn).await?;
    // load(&mut *conn).await?;

    ldconfig(&mut *conn).await?;

    Ok(())
}

async fn load(conn: &mut SqliteConnection) -> Result<()> {
    let ldraw_paths = [
        "/mnt/c/program files/studio 2.0/ldraw/ldconfig.ldr",
        "/mnt/c/users/andrew/documents/lego/rendering/ldraw/ldconfig.ldr",
        "/mnt/c/users/andrew/documents/lego/rendering/ldraw/ldcfgalt.ldr",
    ];
    let ldraw_names = ["studio", "ldraw", "ldraw_alt"];
    for (name, path) in ldraw_names.iter().zip(ldraw_paths) {
        let ldconfig = std::fs::read_to_string(path)?;
        ldraw::insert_file(&ldconfig, name, &mut *conn).await?;
    }

    let studio_paths = [
        "/mnt/c/program files/studio 2.0/data/studiocolordefinition.txt",
        "/mnt/c/program files/studio 2.0/data/customcolordefinition.txt",
        "/mnt/c/program files/studio 2.0/data/customcolors/customcolordefinition.txt",
    ];
    let studio_names = ["studio", "custom1", "custom2"];
    for (name, path) in studio_names.iter().zip(studio_paths) {
        let definitions = std::fs::read_to_string(path)?;
        studio::insert_file(&definitions, name, &mut *conn).await?;
    }

    let eyesight_paths = [
        "/mnt/c/program files/studio 2.0/photorealisticrenderer/win/64/settings.xml",
        "/mnt/c/program files/studio 2.0/data/customcolors/customcolorsettings.xml",
    ];
    let eyesight_names = ["eyesight", "custom", "unpixelled"];
    for (name, path) in eyesight_names.iter().zip(eyesight_paths) {
        let definitions = std::fs::read_to_string(path)?;
        eyesight::insert_file(&definitions, name, &mut *conn).await?;
    }

    Ok(())
}

pub async fn python_dict(conn: &mut SqliteConnection) -> Result<()> {
    let rows = sqlx::query!("SELECT * FROM eyesight_color")
        .fetch_all(&mut *conn)
        .await?;
    let mut map = HashMap::new();
    for row in rows {
        map.insert(row.name, (row.red, row.green, row.blue));
    }

    let rows =
        sqlx::query!("SELECT distinct ldraw_code, studio_name, category_name FROM studio_color")
            .fetch_all(&mut *conn)
            .await?;

    println!("colors = {{");
    for row in rows {
        if matches!(&*row.studio_name, "CurrentColor" | "EdgeColor") {
            continue;
        }

        let prefix = match &*row.category_name {
            "Solid Colors" => "SOLID-",
            "Transparent Colors" => "TRANS-",
            _ => continue,
        };

        let mut name = row.studio_name.replace(['-', ' '], "_").to_uppercase();
        name.insert_str(0, prefix);
        name = name.replace("TRANS-TRANS_", "TRANS-");

        let Some(rgb) = map.get(&name) else {
            println!("    # {name}");
            continue;
        };

        let id = row.ldraw_code;

        println!("    {id}: {rgb:?},");
    }
    println!("}}");

    Ok(())
}

pub async fn ldconfig(conn: &mut SqliteConnection) -> Result<()> {
    let ldraw_colors = sqlx::query!(
        "
        SELECT DISTINCT
            ldraw.name,
            ldraw.code,
            ldraw.edge,
            ldraw.alpha,
            studio.studio_name,
            studio.category_name
        FROM ldraw_color ldraw
            INNER JOIN studio_color studio ON studio.ldraw_code = ldraw.code
        "
    )
    .fetch_all(&mut *conn)
    .await?;

    let eyesight_colors =
        sqlx::query!("SELECT DISTINCT name, red, green, blue FROM eyesight_color")
            .fetch_all(&mut *conn)
            .await?
            .into_iter()
            .map(|record| {
                let [r, g, b] = [record.red, record.green, record.blue].map(|n| (n * 255.0) as u8);
                let color = format!("#{r:02X}{g:02X}{b:02X}");
                (record.name, color)
            })
            .collect::<HashMap<_, _>>();

    for c in &ldraw_colors {
        let Some(category) = c.category_name.as_deref() else {
            continue;
        };
        let prefix = match category {
            "Solid Colors" => "SOLID-",
            "Transparent Colors" => "TRANS-",
            _ => continue,
        };

        let Some(studio_name) = c.studio_name.as_deref() else {
            continue;
        };

        let mut eyesight_name = studio_name
            .trim_start_matches("Trans-")
            .replace(['-', ' '], "_")
            .to_uppercase();

        eyesight_name.insert_str(0, prefix);

        let Some(eyesight_color) = eyesight_colors.get(&eyesight_name) else {
            continue;
        };

        let edge = format!("#{:06X}", c.edge);

        print!(
            "0 !COLOUR {} CODE {} VALUE {} EDGE {}",
            c.name, c.code, eyesight_color, edge
        );
        if let Some(alpha) = c.alpha {
            print!(" ALPHA {alpha}");
        }
        println!();
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
