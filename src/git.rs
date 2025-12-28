use git2::{
    build::CheckoutBuilder, Cred, FetchOptions, PushOptions, RemoteCallbacks, Repository,
    Signature,
};
use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::{TempDir, tempdir};

/// Returns true if a file can be written because it already exists or its parent directory exists.
fn path_editable<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();
    path.is_file() || path.parent().is_some_and(std::path::Path::is_dir)
}

/// Lightweight handle for cloning, reading, and writing to a remote git repository.
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
    /// Clone the remote into a tempdir (kept alive) using SSH and set default identity.
    pub fn init(remote_url: &str) -> Self {
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
        }
    }

    /// Fetch the branch tip, peel to a tree, and return the blob for the given path.
    pub fn read_file(&self, relative_path: &str, branch_name: Option<&str>) -> Option<String> {
        let repo = Repository::open(&self.repo_directory).ok()?;
        let branch_name = branch_name.unwrap_or("prime");
        let _ = fetch_branch(&repo, branch_name);
        let reference = repo
            .find_reference(&format!("refs/remotes/origin/{branch_name}"))
            .or_else(|_| repo.find_reference("refs/remotes/origin/prime"))
            .ok()?;
        let commit = reference.peel_to_commit().ok()?;
        let tree = commit.tree().ok()?;
        let entry = tree.get_path(Path::new(relative_path)).ok()?;
        let blob = repo.find_blob(entry.id()).ok()?;
        std::str::from_utf8(blob.content())
            .ok()
            .map(std::string::ToString::to_string)
    }

    /// Ensure remote is current, check out target branch, write, commit, and push content.
    pub fn write_file(
        &self,
        relative_path: &str,
        content: &str,
        branch_name: Option<&str>,
    ) -> Result<(), ()> {
        let repo = Repository::open(&self.repo_directory).map_err(|_| ())?;
        let branch_name = branch_name.unwrap_or("prime");
        let workdir = repo.workdir().ok_or(())?;

        let _ = fetch_branch(&repo, branch_name);
        let _ = fetch_branch(&repo, "prime");

        let base_reference = repo
            .find_reference(&format!("refs/remotes/origin/{branch_name}"))
            .or_else(|_| repo.find_reference("refs/remotes/origin/prime"))
            .map_err(|_| ())?;
        let base_commit = base_reference.peel_to_commit().map_err(|_| ())?;

        if repo
            .find_branch(branch_name, git2::BranchType::Local)
            .is_err()
        {
            repo.branch(branch_name, &base_commit, true)
                .map_err(|_| ())?;
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

        let target_path = workdir.join(relative_path);
        if !path_editable(&target_path) {
            return Err(());
        }
        std::fs::write(&target_path, content).map_err(|_| ())?;

        let mut index = repo.index().map_err(|_| ())?;
        index.add_path(Path::new(relative_path)).map_err(|_| ())?;
        index.write().map_err(|_| ())?;
        let tree_id = index.write_tree().map_err(|_| ())?;
        let tree = repo.find_tree(tree_id).map_err(|_| ())?;

        let parent_reference = repo
            .find_reference(&format!("refs/heads/{branch_name}"))
            .map_err(|_| ())?;
        let parent_commit = parent_reference.peel_to_commit().map_err(|_| ())?;
        let signature = repo.signature().unwrap_or_else(|_| {
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

        let mut remote = repo.find_remote("origin").map_err(|_| ())?;
        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(ssh_callbacks());
        remote
            .push(
                &[&format!(
                    "refs/heads/{branch_name}:refs/heads/{branch_name}"
                )],
                Some(&mut push_options),
            )
            .map_err(|_| ())?;

        Ok(())
    }
}

/// Build SSH callbacks that always use the app key at $HOME/.ssh/id_ed25519.
fn ssh_callbacks<'cb>() -> RemoteCallbacks<'cb> {
    let mut callbacks = RemoteCallbacks::new();
    let home = env::var("HOME")
        .expect("HOME not set; required to locate SSH key");
    callbacks.credentials(move |_url, username_from_url, _allowed_types| {
        let username = username_from_url.expect("username missing for SSH auth");
        let key_path = Path::new(&home).join(".ssh/id_ed25519");
        Cred::ssh_key(username, None, &key_path, None)
    });
    callbacks
}

/// Fetch a single branch from origin using the shared SSH credentials.
fn fetch_branch(repo: &Repository, branch: &str) -> Result<(), git2::Error> {
    let mut remote = repo.find_remote("origin")?;
    let mut options = FetchOptions::new();
    options.remote_callbacks(ssh_callbacks());
    remote.fetch(&[branch], Some(&mut options), None)
}

#[cfg(test)]
mod tests {
    use super::{GitRemote, fetch_branch, path_editable};
    use git2::{Repository, Signature};
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    /// Seed a worktree commit with the given file content.
    fn write_and_commit(
        repo: &Repository,
        relative_path: &str,
        content: &str,
    ) -> Result<(), git2::Error> {
        let workdir = repo.workdir().expect("workdir missing");
        let full_path = workdir.join(relative_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).expect("create_dir_all failed");
        }
        fs::write(&full_path, content).expect("write failed");

        let mut index = repo.index()?;
        index.add_path(Path::new(relative_path))?;
        index.write()?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        let signature = Signature::now("Tester", "tester@example.com")?;
        let parent = repo.head().ok().and_then(|h| h.target());
        let parents = if let Some(parent_id) = parent {
            vec![repo.find_commit(parent_id)?]
        } else {
            Vec::new()
        };
        let parent_refs: Vec<&git2::Commit> = parents.iter().collect();
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "seed commit",
            &tree,
            &parent_refs,
        )?;
        Ok(())
    }

    /// Create a bare remote with a prime branch containing wiki/index.md.
    fn seed_remote() -> tempfile::TempDir {
        let remote_dir = tempdir().expect("tempdir failed");
        Repository::init_bare(remote_dir.path()).expect("init_bare failed");

        let work_dir = tempdir().expect("workdir tempdir failed");
        let work_repo = Repository::init(work_dir.path()).expect("init work repo failed");
        work_repo
            .remote("origin", remote_dir.path().to_str().unwrap())
            .expect("add remote failed");
        write_and_commit(&work_repo, "wiki/index.md", "hello world").expect("seed commit failed");
        let head_commit = work_repo
            .head()
            .unwrap()
            .peel_to_commit()
            .expect("peel_to_commit failed");
        if work_repo.find_reference("refs/heads/prime").is_err() {
            work_repo
                .branch("prime", &head_commit, false)
                .expect("create prime branch failed");
        }
        work_repo
            .set_head("refs/heads/prime")
            .expect("set_head failed");
        let mut remote = work_repo.find_remote("origin").expect("find_remote failed");
        remote
            .push(&["refs/heads/prime:refs/heads/prime"], None)
            .expect("initial push failed");
        remote_dir
    }

    /// Read a blob from a given branch/path in the provided repository.
    fn read_blob(repo: &Repository, branch: &str, path: &str) -> Option<String> {
        let reference = repo.find_reference(&format!("refs/heads/{branch}")).ok()?;
        let commit = reference.peel_to_commit().ok()?;
        let tree = commit.tree().ok()?;
        let entry = tree.get_path(Path::new(path)).ok()?;
        let blob = repo.find_blob(entry.id()).ok()?;
        std::str::from_utf8(blob.content()).ok().map(str::to_string)
    }

    /// path_editable allows existing files and paths with existing parents.
    #[test]
    fn path_editable_accepts_existing_paths() {
        let tmp = tempdir().unwrap();
        let file_path = tmp.path().join("file.txt");
        fs::write(&file_path, "data").unwrap();
        assert!(path_editable(&file_path));
        let nested = tmp.path().join("dir").join("file.txt");
        fs::create_dir_all(tmp.path().join("dir")).unwrap();
        assert!(path_editable(&nested));
    }

    /// read_file fetches and returns blob content from the specified branch.
    #[test]
    fn read_file_returns_content() {
        let remote_dir = seed_remote();
        let git_remote = GitRemote::init(remote_dir.path().to_str().unwrap());
        let content = git_remote
            .read_file("wiki/index.md", Some("prime"))
            .expect("content missing");
        assert_eq!(content, "hello world");
    }

    /// write_file writes content, commits, and pushes to a new user branch.
    #[test]
    fn write_file_writes_to_new_branch() {
        let remote_dir = seed_remote();
        let git_remote = GitRemote::init(remote_dir.path().to_str().unwrap());
        git_remote
            .write_file("wiki/new.md", "new content", Some("user/alice"))
            .expect("write_file failed");

        let remote_repo =
            Repository::open_bare(remote_dir.path()).expect("open_bare remote failed");
        fetch_branch(&remote_repo, "user/alice").ok();
        let content =
            read_blob(&remote_repo, "user/alice", "wiki/new.md").expect("blob missing in remote");
        assert_eq!(content, "new content");
    }
}
