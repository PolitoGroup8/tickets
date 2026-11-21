

use sqlx::PgPool;


pub async fn db_health(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(pool)
        .await
        .map(|_| ())
}

/// The migrations of `migrations/`, built into the binary (`build.rs` rebuilds it when a file
/// is added there).
pub fn migrator() -> Migrator {
    let mut migrator = sqlx::migrate!();
    // See part B, task 3.
    migrator.set_ignore_missing(true);
    migrator
}

/// The message of a migration that failed, with the detail that PostgreSQL adds to it, if any.
pub fn migration_failure(e: &MigrateError) -> String {
    let MigrateError::ExecuteMigration(sqlx::Error::Database(db), version) = e else {
        return e.to_string();
    };
    let detail = db.try_downcast_ref::<PgDatabaseError>().and_then(|pg| pg.detail()).unwrap_or("");
    format!("migration {version}: {}\n{detail}", db.message())
}

/// The version of the last migration applied to the database.
pub async fn schema_version(pool: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT coalesce(max(version), 0) FROM _sqlx_migrations WHERE success")
        .fetch_one(pool)
        .await
}