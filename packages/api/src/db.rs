use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};
use std::str::FromStr;

const INITIAL_MEMBERS: &[&str] = &[
    "Swexbe", "Nox", "Karro", "Vidde", "Stasia", "Dino", "Yoda", "Carl", "Arvid",
];

pub async fn init_pool(db_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(db_url)?.create_if_missing(true);

    let pool = SqlitePoolOptions::new().connect_with(options).await?;

    tracing::info!("Running database migrations");
    sqlx::migrate!().run(&pool).await?;
    tracing::info!("Migrations complete");

    seed_members(&pool).await?;

    Ok(pool)
}

async fn seed_members(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM members")
        .fetch_one(pool)
        .await?;

    if count == 0 {
        for (i, member) in INITIAL_MEMBERS.iter().enumerate() {
            sqlx::query("INSERT OR IGNORE INTO members (name, sort_order) VALUES (?, ?)")
                .bind(member)
                .bind(i as i64)
                .execute(pool)
                .await?;
        }
    }

    Ok(())
}
