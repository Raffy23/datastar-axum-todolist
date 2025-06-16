use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use tracing::info;

use crate::utils;

pub(crate) const DB_URL: &'static str = "sqlite://sqlite.db";

pub async fn create_pool() -> SqlitePool {
    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        info!("Creating database {}", DB_URL);

        match Sqlite::create_database(DB_URL).await {
            Ok(_) => info!("Creating DB was successful"),
            Err(error) => panic!("error: {}", error),
        }
    } else {
        info!("Database already exists");
    }

    let db = SqlitePool::connect(DB_URL).await.unwrap();

    let server_dir = utils::server_directory();
    let migrations = std::path::Path::new(&server_dir).join("./migrations");

    let migration_results = sqlx::migrate::Migrator::new(migrations)
        .await
        .unwrap()
        .run(&db)
        .await;

    match migration_results {
        Ok(_) => info!("Migration success"),
        Err(error) => {
            panic!("error: {}", error);
        }
    }

    db
}
