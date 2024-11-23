use diesel::sqlite::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use log::{error, info};
use diesel_migrations::MigrationHarness; // 添加此行以引入 MigrationHarness trait


pub async fn run_db_migrations(conn: &mut SqliteConnection) {
    const MIGRATIONS: EmbeddedMigrations = embed_migrations!();
    if let Err(err) = conn.run_pending_migrations(MIGRATIONS) {
        error!("Error running migrations: {}", err);
    } else {
        info!("Database migrations executed successfully");
    }
} 