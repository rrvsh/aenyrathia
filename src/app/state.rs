use super::settings::AppSettings;
use crate::git::GitRemote;

#[derive(Clone)]
pub struct AppState {
    pub remote: GitRemote,
}

impl AppState {
    pub fn init(settings: &AppSettings) -> Self {
        Self {
            remote: GitRemote::init(&settings.git_remote),
        }
    }
}
