use std::path::Path;

use sqlx::{sqlite::SqliteConnectOptions, Acquire, SqlitePool};

pub mod ldraw;
pub mod studio;

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

    let x = include_str!("/mnt/c/program files/studio 2.0 earlyaccess/ldraw/ldconfig.ldr");
    ldraw::insert_file(x, "studio earlyaccess", &mut *conn).await?;

    let y =
        include_str!("/mnt/c/program files/studio 2.0 earlyaccess/data/studiocolordefinition.txt");
    studio::insert_file(y, "studio earlyaccess", &mut *conn).await?;

    println!("done");

    Ok(())
}

fn parse_hex(s: &str) -> Option<u32> {
    let s = s.strip_prefix("#")?;
    if s.len() != 6 {
        return None;
    }
    u32::from_str_radix(s, 0x10).ok()
}
