use std::path::Path;

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

    // let ldraw_paths = [
    //     "/mnt/c/program files/studio 2.0/ldraw/ldconfig.ldr",
    //     "/mnt/c/users/andrew/documents/lego/rendering/ldraw/ldconfig.ldr",
    //     "/mnt/c/users/andrew/documents/lego/rendering/ldraw/ldcfgalt.ldr",
    // ];
    // let ldraw_names = ["studio", "ldraw", "ldraw_alt"];
    // for (name, path) in ldraw_names.iter().zip(ldraw_paths) {
    //     let ldconfig = std::fs::read_to_string(path)?;
    //     ldraw::insert_file(&ldconfig, name, &mut *conn).await?;
    // }

    // let studio_paths = [
    //     "/mnt/c/program files/studio 2.0/data/studiocolordefinition.txt",
    //     "/mnt/c/program files/studio 2.0/data/customcolordefinition.txt",
    //     "/mnt/c/program files/studio 2.0/data/customcolors/customcolordefinition.txt",
    // ];
    // let studio_names = ["studio", "custom1", "custom2"];
    // for (name, path) in studio_names.iter().zip(studio_paths) {
    //     let definitions = std::fs::read_to_string(path)?;
    //     studio::insert_file(&definitions, name, &mut *conn).await?;
    // }

    let eyesight_paths = [
        "/mnt/c/program files/studio 2.0/photorealisticrenderer/win/64/settings.xml",
        "/mnt/c/program files/studio 2.0/data/customcolors/customcolorsettings.xml",
    ];
    let eyesight_names = ["eyesight"];
    for (name, path) in eyesight_names.iter().zip(eyesight_paths) {
        let definitions = std::fs::read_to_string(path)?;
        eyesight::insert_file(&definitions, name, &mut *conn).await?;
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
