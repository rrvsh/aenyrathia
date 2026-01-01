pub struct AppSettings {
    pub addr: String,
    pub git_remote: String,
    pub git_sync_interval_ms: u64,
    pub db_url: String,
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
        let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://username:password@localhost:54322/default_database".to_string()
        });

        Self {
            git_remote,
            addr: format!("{host}:{port}"),
            git_sync_interval_ms,
            db_url,
        }
    }
}
