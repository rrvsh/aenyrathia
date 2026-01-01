pub struct AppSettings {
    pub addr: String,
    pub git_remote: String,
    pub git_sync_interval_ms: u64,
    pub db_options: sqlx::postgres::PgConnectOptions,
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
        let db_host = std::env::var("PGHOST").unwrap_or_else(|_| {
            "/cloudsql/aenyrathia:asia-southeast1:aenyrathia-pgsql".to_string()
        });
        let db_port = std::env::var("PGPORT")
            .ok()
            .and_then(|v| v.parse::<u16>().ok())
            .unwrap_or(5432);
        let db_user = std::env::var("PGUSER").unwrap_or_else(|_| "rafiq".to_string());
        let db_password = std::env::var("PGPASSWORD").unwrap_or_else(|_| "1234".to_string());
        let db_name = std::env::var("PGDATABASE").unwrap_or_else(|_| "app_db".to_string());
        let db_options = sqlx::postgres::PgConnectOptions::new()
            .host(&db_host)
            .port(db_port)
            .username(&db_user)
            .password(&db_password)
            .database(&db_name);

        Self {
            git_remote,
            addr: format!("{host}:{port}"),
            git_sync_interval_ms,
            db_options,
        }
    }
}
