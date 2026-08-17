use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::error::ApiError;

pub const MIGRATIONS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/migrations");

pub async fn connect(database_url: &str) -> Result<PgPool, ApiError> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    run(&pool).await?;
    Ok(pool)
}

pub async fn health_check(pool: &PgPool) -> Result<(), ApiError> {
    sqlx::query("SELECT 1").execute(pool).await?;
    Ok(())
}

/// Apply pending sqlx schema migrations. Demo rows are not included; use `seed`.
pub async fn run(pool: &PgPool) -> Result<(), ApiError> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}

const DEMO_SQL: &str = include_str!("../fixtures/demo.sql");

/// Insert local demo index rows. Idempotent. Local settlement DB only.
pub async fn seed(pool: &PgPool, database_url: &str) -> Result<(), ApiError> {
    if !is_local_settlement_db(database_url) {
        return Err(ApiError::Config(
            "seed is allowed only against a local settlement database".into(),
        ));
    }
    for statement in DEMO_SQL.split(';') {
        let sql = statement
            .lines()
            .filter(|line| !line.trim_start().starts_with("--"))
            .collect::<Vec<_>>()
            .join("\n");
        let sql = sql.trim();
        if sql.is_empty() {
            continue;
        }
        sqlx::query(sql).execute(pool).await?;
    }
    Ok(())
}

pub fn is_local_settlement_db(database_url: &str) -> bool {
    let Ok(parsed) = url::Url::parse(database_url) else {
        return false;
    };
    let Some(host) = parsed.host_str() else {
        return false;
    };
    let host_ok = matches!(host, "localhost" | "127.0.0.1" | "::1");
    let db_name = parsed.path().trim_start_matches('/');
    host_ok && db_name == "settlement"
}

pub fn reset_is_allowed(app_env: &str, database_url: &str) -> bool {
    matches!(app_env, "development" | "dev" | "local") && is_local_settlement_db(database_url)
}

#[derive(Debug, Clone)]
pub struct AppliedMigration {
    pub version: i64,
    pub description: String,
    pub success: bool,
}

#[derive(Debug, Clone)]
pub struct LocalMigration {
    pub version: i64,
    pub description: String,
    pub filename: String,
}

pub async fn applied(pool: &PgPool) -> Result<Vec<AppliedMigration>, ApiError> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1 FROM information_schema.tables
            WHERE table_schema = 'public' AND table_name = '_sqlx_migrations'
        )",
    )
    .fetch_one(pool)
    .await?;

    if !exists {
        return Ok(Vec::new());
    }

    let rows = sqlx::query_as::<_, (i64, String, bool)>(
        "SELECT version, description, success FROM _sqlx_migrations ORDER BY version",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(version, description, success)| AppliedMigration {
            version,
            description,
            success,
        })
        .collect())
}

pub fn local_migrations() -> Result<Vec<LocalMigration>, ApiError> {
    local_migrations_in(Path::new(MIGRATIONS_DIR))
}

pub fn local_migrations_in(dir: &Path) -> Result<Vec<LocalMigration>, ApiError> {
    let mut rows = Vec::new();
    let entries = fs::read_dir(dir).map_err(|e| ApiError::Config(e.to_string()))?;
    for entry in entries {
        let entry = entry.map_err(|e| ApiError::Config(e.to_string()))?;
        let filename = entry.file_name().to_string_lossy().into_owned();
        if !filename.ends_with(".sql") {
            continue;
        }
        let Some(version) = parse_version(&filename) else {
            continue;
        };
        let description = filename
            .trim_end_matches(".sql")
            .split_once('_')
            .map(|(_, rest)| rest.replace('_', " "))
            .unwrap_or_default();
        rows.push(LocalMigration {
            version,
            description,
            filename,
        });
    }
    rows.sort_by_key(|row| row.version);
    Ok(rows)
}

pub fn parse_version(filename: &str) -> Option<i64> {
    let stem = filename.strip_suffix(".sql")?;
    let digits = stem.split_once('_').map(|(head, _)| head).unwrap_or(stem);
    digits.parse().ok()
}

pub fn slugify(name: &str) -> String {
    let slug = name
        .trim()
        .to_ascii_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>();
    slug.trim_matches('_')
        .split('_')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

pub fn next_migration_path(dir: &Path, name: &str) -> Result<PathBuf, ApiError> {
    let slug = slugify(name);
    if slug.is_empty() {
        return Err(ApiError::Config(
            "migration name must contain letters or digits".into(),
        ));
    }
    let next = local_migrations_in(dir)?
        .into_iter()
        .map(|row| row.version)
        .max()
        .unwrap_or(0)
        + 1;
    Ok(dir.join(format!("{next:04}_{slug}.sql")))
}

pub fn add_migration(name: &str) -> Result<PathBuf, ApiError> {
    let path = next_migration_path(Path::new(MIGRATIONS_DIR), name)?;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|e| ApiError::Config(e.to_string()))?;
    file.write_all(b"-- Write the SQL for this migration. sqlx applies files in version order.\n")
        .map_err(|e| ApiError::Config(e.to_string()))?;
    Ok(path)
}

/// Dev-only: drop the public schema and re-apply all migrations.
/// Opens its own pool against the validated URL so a mismatched pool cannot be dropped.
pub async fn reset(database_url: &str) -> Result<PgPool, ApiError> {
    let app_env = std::env::var("APP_ENV").unwrap_or_default();
    if !reset_is_allowed(&app_env, database_url) {
        return Err(ApiError::Config(
            "reset requires APP_ENV=development and a local settlement DATABASE_URL".into(),
        ));
    }
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(database_url)
        .await?;
    sqlx::query("DROP SCHEMA IF EXISTS public CASCADE")
        .execute(&pool)
        .await?;
    sqlx::query("CREATE SCHEMA public").execute(&pool).await?;
    run(&pool).await?;
    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_version_reads_numeric_prefix() {
        assert_eq!(parse_version("0002_proofs.sql"), Some(2));
        assert!(parse_version("README.md").is_none());
    }

    #[test]
    fn slugify_normalizes_names() {
        assert_eq!(slugify("Seed Demo"), "seed_demo");
        assert_eq!(slugify("  add-index!!  "), "add_index");
    }

    #[test]
    fn reset_guard_rejects_remote_urls() {
        assert!(!reset_is_allowed(
            "development",
            "postgres://settlement:settlement@db.example.com:5432/settlement"
        ));
        assert!(!reset_is_allowed(
            "production",
            "postgres://settlement:settlement@127.0.0.1:5433/settlement"
        ));
        assert!(!is_local_settlement_db(
            "postgres://user:127.0.0.1@db.example.com:5432/settlement"
        ));
        assert!(!is_local_settlement_db(
            "postgres://settlement:settlement@127.0.0.1:5433/other"
        ));
        assert!(reset_is_allowed(
            "development",
            "postgres://settlement:settlement@127.0.0.1:5433/settlement"
        ));
        assert!(is_local_settlement_db(
            "postgres://settlement:settlement@localhost:5433/settlement"
        ));
    }

    #[test]
    fn next_migration_path_increments() {
        let dir = Path::new(MIGRATIONS_DIR);
        let path = next_migration_path(dir, "later_change").unwrap();
        let name = path.file_name().unwrap().to_string_lossy();
        assert!(name.ends_with("_later_change.sql"), "{name}");
        let version = parse_version(&name).unwrap();
        let max_existing = local_migrations_in(dir)
            .unwrap()
            .into_iter()
            .map(|row| row.version)
            .max()
            .unwrap();
        assert_eq!(version, max_existing + 1);
    }
}
