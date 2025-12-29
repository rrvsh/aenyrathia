use super::settings::AppSettings;
use crate::git::GitRemote;
use std::time::Duration;

#[derive(Clone)]
pub struct AppState {
    pub remote: GitRemote,
}

impl AppState {
    pub fn init(settings: &AppSettings) -> Self {
        Self {
            remote: GitRemote::init_with_interval(
                &settings.git_remote,
                Duration::from_millis(settings.git_sync_interval_ms),
            ),
        }
    }
}
