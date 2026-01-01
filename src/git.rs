use git2::{
    BranchType, Cred, FetchOptions, PushOptions, RemoteCallbacks, Repository, Signature,
    build::CheckoutBuilder,
};
use log::{trace, warn};
use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tempfile::{TempDir, tempdir};

/// Returns true if a file can be written because it already exists or its parent directory exists.
fn path_editable<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();
    path.is_file() || path.parent().is_some_and(std::path::Path::is_dir)
}

/// Author information for a commit.
pub struct Author {
    pub name: String,
    pub email: String,
}

/// Lightweight handle for cloning, reading, and writing to a remote git repository.
pub struct GitRemote {
    /// Keep tempdir alive so the cloned repo is not deleted.
    tempdir: Arc<TempDir>,
    repo_directory: PathBuf,
    sync_interval: Duration,
}

impl Clone for GitRemote {
    fn clone(&self) -> Self {
        Self {
            tempdir: Arc::clone(&self.tempdir),
            repo_directory: self.repo_directory.clone(),
            sync_interval: self.sync_interval,
        }
    }
}

impl GitRemote {
    /// Clone the remote into a tempdir (kept alive) using SSH and set default identity with a custom sync interval.
    pub fn init_with_interval(remote_url: &str, sync_interval: Duration) -> Self {
        let tempdir = tempdir().expect("Error creating tempdir!");
        let mut builder = git2::build::RepoBuilder::new();
        let mut fo = FetchOptions::new();
        fo.remote_callbacks(ssh_callbacks());
        builder.fetch_options(fo);
        let repository = builder
            .clone(remote_url, tempdir.path())
            .expect("Error cloning repository!");

        if let Ok(mut repository_config) = repository.config() {
            let _ = repository_config.set_str("user.name", "aenyrathia.wiki");
            let _ = repository_config.set_str("user.email", "git@aenyrathia.wiki");
        }

        Self {
            tempdir: Arc::new(tempdir),
            repo_directory: repository
                .workdir()
                .unwrap_or_else(|| repository.path())
                .to_path_buf(),
            sync_interval,
        }
        .with_background_sync()
    }

    /// Fetch the branch tip, peel to a tree, and return the blob for the given path.
    pub fn read_file(&self, relative_path: &str, branch_name: Option<&str>) -> Option<String> {
        let repo = Repository::open(&self.repo_directory).ok()?;
        let branch_name = branch_name.unwrap_or("prime");
        let reference = repo
            .find_reference(&format!("refs/remotes/origin/{branch_name}"))
            .or_else(|_| repo.find_reference("refs/remotes/origin/prime"))
            .ok()?;
        let commit = reference.peel_to_commit().ok()?;
        trace!(
            "latest commit summary on ref {:?}: {:?}",
            reference.name(),
            commit.summary()
        );
        let tree = commit.tree().ok()?;
        trace!(
            "latest tree id on ref {:?}: {:?}",
            reference.name(),
            tree.id()
        );
        let entry = tree.get_path(Path::new(relative_path)).ok()?;
        trace!(
            "tree entry name {:?} for path {relative_path} on ref {:?}",
            entry.name(),
            reference.name()
        );

        let blob = repo.find_blob(entry.id()).ok()?;
        let blob_content = std::str::from_utf8(blob.content()).ok()?;
        trace!(
            "blob content {blob_content} for path {relative_path} on ref {:?}",
            reference.name()
        );
        Some(blob_content.to_string())
    }

    /// Ensure remote is current, check out target branch, write, commit, and push content.
    pub fn write_file(
        &self,
        relative_path: &str,
        content: &str,
        branch_name: Option<&str>,
        author: Option<&Author>,
    ) -> Result<(), ()> {
        let repo = Repository::open(&self.repo_directory).map_err(|_| ())?;
        let branch_name = branch_name.unwrap_or("prime");

        let base_reference = repo
            .find_reference(&format!("refs/remotes/origin/{branch_name}"))
            .or_else(|_| repo.find_reference("refs/remotes/origin/prime"))
            .map_err(|_| ())?;
        let base_commit = base_reference.peel_to_commit().map_err(|_| ())?;
        let target_oid = base_commit.id();
        match repo.find_branch(branch_name, git2::BranchType::Local) {
            Ok(local_branch) => {
                local_branch
                    .into_reference()
                    .set_target(target_oid, "force update from remote")
                    .map_err(|_| ())?;
            }
            Err(_) => {
                repo.branch(branch_name, &base_commit, true)
                    .map_err(|_| ())?;
            }
        }
        repo.set_head(&format!("refs/heads/{branch_name}"))
            .map_err(|_| ())?;
        repo.checkout_head(Some(
            CheckoutBuilder::new()
                .force()
                .remove_untracked(true)
                .allow_conflicts(false),
        ))
        .map_err(|_| ())?;
        trace!(
            "Checked out base commit {:?} for branch refs/heads/{branch_name}.",
            base_commit.summary()
        );

        let workdir = repo.workdir().ok_or(())?;
        let target_path = workdir.join(relative_path);
        if !path_editable(&target_path) {
            return Err(());
        }
        std::fs::write(&target_path, content).map_err(|_| ())?;
        trace!(
            "Wrote {content} to file at path {} for branch refs/heads/{branch_name}.",
            target_path.display()
        );

        let mut index = repo.index().map_err(|_| ())?;
        index.add_path(Path::new(relative_path)).map_err(|_| ())?;
        index.write().map_err(|_| ())?;
        let tree_id = index.write_tree().map_err(|_| ())?;
        let tree = repo.find_tree(tree_id).map_err(|_| ())?;
        let parent_reference = repo
            .find_reference(&format!("refs/heads/{branch_name}"))
            .map_err(|_| ())?;
        let parent_commit = parent_reference.peel_to_commit().map_err(|_| ())?;
        let signature = author
            .and_then(|author| Signature::now(&author.name, &author.email).ok())
            .or_else(|| repo.signature().ok())
            .unwrap_or_else(|| {
                Signature::now("aenyrathia.wiki", "git@aenyrathia.wiki")
                    .expect("Failed to create fallback signature")
            });
        repo.commit(
            Some(&format!("refs/heads/{branch_name}")),
            &signature,
            &signature,
            &format!("Update {relative_path}"),
            &tree,
            &[&parent_commit],
        )
        .map_err(|_| ())?;
        trace!("Committed to branch refs/heads/{branch_name}.");

        Ok(())
    }

    /// Start a background worker that periodically fetches and pushes all branches with backoff on failures.
    fn with_background_sync(self) -> Self {
        let repo_directory = self.repo_directory.clone();
        let tempdir = Arc::clone(&self.tempdir);
        let interval = self.sync_interval;
        thread::Builder::new()
            .name("git-sync-worker".to_string())
            .spawn(move || {
                // Keep the tempdir alive for the lifetime of the worker.
                let _keep_alive = tempdir;
                let max_backoff = Duration::from_secs(30);
                let mut backoff = interval;
                loop {
                    match sync_once(&repo_directory) {
                        Ok(()) => {
                            backoff = interval;
                            thread::sleep(interval);
                        }
                        Err(err) => {
                            warn!("Git background sync failed: {err}");
                            thread::sleep(backoff);
                            backoff = std::cmp::min(backoff * 2, max_backoff);
                        }
                    }
                }
            })
            .expect("Failed to start git sync worker");
        self
    }
}

/// Build SSH callbacks that always use the app key at `$HOME/.ssh/id_ed25519`.
fn ssh_callbacks<'cb>() -> RemoteCallbacks<'cb> {
    let mut callbacks = RemoteCallbacks::new();
    let home = env::var("HOME").expect("HOME not set; required to locate SSH key");
    callbacks.credentials(move |_url, username_from_url, _allowed_types| {
        let username = username_from_url.expect("username missing for SSH auth");
        let key_path = Path::new(&home).join(".ssh/id_ed25519");
        Cred::ssh_key(username, None, &key_path, None)
    });
    callbacks
}

/// Fetch all refs and push any local branches that are ahead of their remote tracking refs.
fn sync_once(repo_directory: &Path) -> Result<(), git2::Error> {
    let repo = Repository::open(repo_directory)?;
    fetch_all(&repo)?;
    push_all_branches(&repo)?;
    Ok(())
}

fn fetch_all(repo: &Repository) -> Result<(), git2::Error> {
    let mut remote = repo.find_remote("origin")?;
    let mut options = FetchOptions::new();
    options.remote_callbacks(ssh_callbacks());
    trace!("Background fetch: starting");
    let refspecs: &[&str] = &["refs/heads/*:refs/remotes/origin/*"];
    let result = remote.fetch(refspecs, Some(&mut options), None);
    trace!("Background fetch: done");
    result
}

fn push_all_branches(repo: &Repository) -> Result<(), git2::Error> {
    let mut remote = repo.find_remote("origin")?;
    let mut push_options = PushOptions::new();
    push_options.remote_callbacks(ssh_callbacks());
    for branch_result in repo.branches(Some(BranchType::Local))? {
        let (branch, _) = branch_result?;
        let branch_name = branch
            .name()?
            .ok_or_else(|| git2::Error::from_str("Invalid branch name"))?
            .to_string();

        let reference = branch.into_reference();
        let local_commit = reference.peel_to_commit()?;
        let remote_ref = repo.find_reference(&format!("refs/remotes/origin/{branch_name}"));
        let push_needed = match remote_ref {
            Ok(remote_reference) => {
                if let Ok(remote_commit) = remote_reference.peel_to_commit() {
                    let (ahead, _) =
                        repo.graph_ahead_behind(local_commit.id(), remote_commit.id())?;
                    ahead > 0
                } else {
                    true
                }
            }
            Err(_) => true, // no remote branch yet; create it
        };

        if !push_needed {
            continue;
        }

        trace!("Background push: pushing branch {branch_name}");
        remote.push(
            &[&format!(
                "refs/heads/{branch_name}:refs/heads/{branch_name}"
            )],
            Some(&mut push_options),
        )?;
        trace!("Background push: completed branch {branch_name}");
    }
    Ok(())
}
