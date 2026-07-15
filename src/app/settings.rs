use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous};
use std::path::Path;
use std::str::FromStr;

pub struct AppSettings {
    pub addr: String,
    pub git_remote: String,
    pub git_sync_interval_ms: u64,
    pub db_options: SqliteConnectOptions,
}

impl AppSettings {
    pub fn from_env() -> Self {
        let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
        let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let git_remote = std::env::var("GIT_REMOTE")
            .unwrap_or_else(|_| "git@github.com:rrvsh/aenyrathia.git".to_string());
        let git_sync_interval_ms = std::env::var("GIT_SYNC_INTERVAL_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(1000);
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| "data".to_string());
            std::fs::create_dir_all(&data_dir)
                .unwrap_or_else(|err| panic!("failed to create data dir `{data_dir}`: {err}"));
            let db_path = Path::new(&data_dir).join("aenyrathia.sqlite3");
            format!("sqlite://{}", db_path.display())
        });
        let db_options = SqliteConnectOptions::from_str(&database_url)
            .unwrap_or_else(|err| panic!("invalid DATABASE_URL `{database_url}`: {err}"))
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal);

        Self {
            git_remote,
            addr: format!("{host}:{port}"),
            git_sync_interval_ms,
            db_options,
        }
    }
}
