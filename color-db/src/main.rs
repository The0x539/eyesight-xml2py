use std::{collections::HashMap, path::Path};

use sqlx::{sqlite::SqliteConnectOptions, Acquire, SqlitePool};

pub mod ldraw;

pub mod studio;

pub mod eyesight;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("colors.sqlite");
    let _ = std::fs::remove_file(&path);

    let options = SqliteConnectOptions::new()
        .filename(&path)
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await?;
    let mut pool_conn = pool.acquire().await?;
    let conn = pool_conn.acquire().await?;

    sqlx::migrate!("./migrations").run(&mut *conn).await?;

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

        let mut name = row
            .studio_name
            .replace("-", "_")
            .replace(" ", "_")
            .to_uppercase();

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

fn parse_hex(s: &str) -> Option<u32> {
    let s = s.strip_prefix("#")?;
    if s.len() != 6 {
        return None;
    }
    u32::from_str_radix(s, 0x10).ok()
}
