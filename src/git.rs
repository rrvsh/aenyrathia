use git2::{Cred, RemoteCallbacks, Repository};
use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::{TempDir, tempdir};

/// Checks if a file path is editable by checking if any of the following conditions are true:
/// 1. The file already exists and is not a directory.
/// 2. The parent directory exists.
fn path_editable<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();
    path.is_file() || path.parent().is_some_and(std::path::Path::is_dir)
}

pub struct GitRemote {
    /// Keep tempdir alive so the cloned repo is not deleted.
    tempdir: Arc<TempDir>,
    repo_directory: PathBuf,
}

impl Clone for GitRemote {
    fn clone(&self) -> Self {
        Self {
            tempdir: Arc::clone(&self.tempdir),
            repo_directory: self.repo_directory.clone(),
        }
    }
}

impl GitRemote {
    pub fn init(remote_url: &str) -> Self {
        let tempdir = tempdir().expect("Error creating tempdir!");
        // TODO: use git2 to clone `remote_url` into `repo_directory`
        // and set user.name/user.email on the repo config.
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            Cred::ssh_key(
                username_from_url.unwrap(),
                None,
                Path::new(&format!("{}/.ssh/id_ed25519", env::var("HOME").unwrap())),
                None,
            )
        });
        let mut fo = git2::FetchOptions::new();
        fo.remote_callbacks(callbacks);
        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fo);
        let repository = builder
            .clone(remote_url, tempdir.path())
            .expect("Error cloning repository!");
        let mut repository_config = repository
            .config()
            .expect("Error getting config for repository!");
        let _ = repository_config.set_str("user.name", "aenyrathia.wiki");
        let _ = repository_config.set_str("user.email", "git@aenyrathia.wiki");

        Self {
            tempdir: Arc::new(tempdir),
            repo_directory: repository.path().parent().unwrap().to_path_buf(),
        }
    }

    pub fn read_file(&self, relative_path: &str, branch_name: Option<&str>) -> Option<String> {
        let repo = Repository::open(&self.repo_directory).ok()?;
        let path = repo.commondir().join(relative_path);
        let branch_name = branch_name.unwrap_or("prime");
        // TODO:
        // - fetch `branch_name` from origin if stale
        // - read blob for `relative_path` at branch tip (no worktree checkout)
        // - return contents as String
        let _ = (branch_name, path);
        None
    }

    pub fn write_file(
        &self,
        relative_path: &str,
        content: &str,
        branch_name: Option<&str>,
    ) -> Result<(), ()> {
        let repo = Repository::open(&self.repo_directory).map_err(|_| ())?;
        let path = repo.commondir().join(relative_path);
        let branch_name = branch_name.unwrap_or("prime");
        if path_editable(&path) {
            // TODO:
            // - fetch `branch_name` from origin if stale
            // - check out/update worktree for `branch_name` (or apply tree edit in-memory)
            // - write `content` to `relative_path` (create parents as needed)
            // - stage and commit with a meaningful message/author
            // - push `branch_name` to origin
            let _ = (branch_name, content);
            Ok(())
        } else {
            Err(())
        }
    }
}
