use anyhow::Context;
use settlement_api::db;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::from_path(concat!(env!("CARGO_MANIFEST_DIR"), "/../.env"));
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "up".into());

    match command.as_str() {
        "up" | "migrate" => up().await,
        "status" => status().await,
        "seed" => seed().await,
        "add" => {
            let name = args
                .next()
                .context("usage: migrate add <name> (example: migrate add add_settlement_index)")?;
            let path = db::add_migration(&name).map_err(|e| anyhow::anyhow!(e))?;
            println!("created {}", path.display());
            Ok(())
        }
        "reset" => {
            let confirm = args.next();
            if confirm.as_deref() != Some("--yes") {
                anyhow::bail!("refusing to reset; re-run with: migrate reset --yes");
            }
            reset().await
        }
        "help" | "-h" | "--help" => {
            print_help();
            Ok(())
        }
        other => anyhow::bail!("unknown command '{other}'\n\n{}", HELP),
    }
}

const HELP: &str = "\
SettlementOracle DB migration manager (sqlx)

  cargo run --manifest-path api/Cargo.toml --bin migrate -- [command]

Commands:
  up              Apply pending schema migrations (default; also runs on API boot)
  seed            Load local demo index rows (api/fixtures/demo.sql)
  status          Show local files vs applied versions
  add <name>      Create the next api/migrations/00XX_name.sql
  reset --yes     Drop public schema and re-apply (APP_ENV=development + local DB)
  help            Show this text
";

fn print_help() {
    print!("{HELP}");
}

async fn pool() -> anyhow::Result<(sqlx::PgPool, String)> {
    let database_url = std::env::var("DATABASE_URL")
        .context("DATABASE_URL is required (copy .env.example to .env)")?;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .context("connect postgres")?;
    Ok((pool, database_url))
}

async fn up() -> anyhow::Result<()> {
    let (pool, _) = pool().await?;
    db::run(&pool).await.map_err(|e| anyhow::anyhow!(e))?;
    println!("migrations applied");
    print_status(&pool).await
}

async fn seed() -> anyhow::Result<()> {
    let (pool, database_url) = pool().await?;
    db::run(&pool).await.map_err(|e| anyhow::anyhow!(e))?;
    db::seed(&pool, &database_url)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    println!("demo seed applied");
    Ok(())
}

async fn status() -> anyhow::Result<()> {
    let (pool, _) = pool().await?;
    print_status(&pool).await
}

async fn print_status(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    let local = db::local_migrations().map_err(|e| anyhow::anyhow!(e))?;
    let applied = db::applied(pool).await.map_err(|e| anyhow::anyhow!(e))?;
    println!("{:<8} {:<10} {}", "VERSION", "STATE", "DESCRIPTION");
    for file in &local {
        let state = applied
            .iter()
            .find(|row| row.version == file.version)
            .map(|row| if row.success { "applied" } else { "failed" })
            .unwrap_or("pending");
        println!("{:<8} {:<10} {}", file.version, state, file.description);
    }
    let pending = local
        .iter()
        .filter(|file| applied.iter().all(|row| row.version != file.version))
        .count();
    println!("{pending} pending");
    Ok(())
}

async fn reset() -> anyhow::Result<()> {
    let database_url = std::env::var("DATABASE_URL")
        .context("DATABASE_URL is required (copy .env.example to .env)")?;
    let pool = db::reset(&database_url)
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    println!("schema reset and migrations re-applied");
    print_status(&pool).await
}
