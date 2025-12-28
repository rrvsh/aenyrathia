use log::trace;
use std::path::{Path, PathBuf};
use std::process::Command;
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
        let repo_directory = tempdir.path().to_path_buf();
        trace!(
            "`git clone {remote_url} {}`: {:?}",
            repo_directory.display(),
            std::process::Command::new("git")
                .args([
                    "clone",
                    remote_url,
                    repo_directory
                        .to_str()
                        .expect("Invalid UTF-8 in tempdir path!"),
                ])
                .output()
                .expect("git command failed to start")
        );
        trace!(
            "`git config user.name aenyrathia.wiki`: {:?}",
            std::process::Command::new("git")
                .current_dir(&repo_directory)
                .args(["config", "user.name", "aenyrathia.wiki"])
                .output()
                .expect("git command failed to start")
        );
        trace!(
            "`git config user.email git@aenyrathia.wiki`: {:?}",
            std::process::Command::new("git")
                .current_dir(&repo_directory)
                .args(["config", "user.email", "git@aenyrathia.wiki"])
                .output()
                .expect("git command failed to start")
        );
        Self {
            tempdir: Arc::new(tempdir),
            repo_directory,
        }
    }

    pub fn read_file(&self, relative_path: &str, branch_name: Option<&str>) -> Option<String> {
        let branch_name = branch_name.unwrap_or("prime");
        self.checkout_remote_branch(branch_name);
        let path = PathBuf::from(&self.repo_directory).join(relative_path);
        std::fs::read_to_string(&path).ok()
    }

    pub fn write_file(
        &self,
        relative_path: &str,
        content: &str,
        branch_name: Option<&str>,
    ) -> Result<(), ()> {
        let branch_name = branch_name.unwrap_or("prime");
        self.checkout_remote_branch(branch_name);
        let path = PathBuf::from(&self.repo_directory).join(relative_path);
        if path_editable(&path) {
            std::fs::write(&path, content).map_or_else(
                |_| Err(()),
                |()| {
                    self.update_and_sync(branch_name);
                    Ok(())
                },
            )
        } else {
            Err(())
        }
    }

    fn update_and_sync(&self, branch_name: &str) {
        trace!("Rebasing {branch_name} and syncing latest changes to origin.");
        trace!(
            "`git fetch origin`: {:?}",
            Command::new("git")
                .current_dir(&self.repo_directory)
                .args(["fetch", "origin"])
                .output()
                .expect("error running git command")
        );
        trace!(
            "`git add .`: {:?}",
            Command::new("git")
                .current_dir(&self.repo_directory)
                .args(["add", "."])
                .output()
                .expect("error running git command")
        );
        trace!(
            "`git commit -m \"test\"`: {:?}",
            Command::new("git")
                .current_dir(&self.repo_directory)
                .args(["commit", "-m", "test"])
                .output()
                .expect("error running git command")
        );
        trace!(
            "`git rebase origin/prime`: {:?}",
            Command::new("git")
                .current_dir(&self.repo_directory)
                .args(["rebase", "origin/prime"])
                .output()
                .expect("error running git command")
        );
        trace!(
            "`git push origin {branch_name}`: {:?}",
            Command::new("git")
                .current_dir(&self.repo_directory)
                .args(["push", "origin", branch_name])
                .output()
                .expect("error running git command")
        );
    }

    fn checkout_remote_branch(&self, branch_name: &str) {
        trace!("Checking out remote branch origin/{branch_name}.");
        trace!(
            "`git fetch origin`: {:?}",
            Command::new("git")
                .current_dir(&self.repo_directory)
                .args(["fetch", "origin"])
                .output()
                .expect("error running git command")
        );
        trace!(
            "`git checkout -B {branch_name}`: {:?}",
            Command::new("git")
                .current_dir(&self.repo_directory)
                .args(["checkout", "-B", branch_name])
                .output()
                .expect("error running git command")
        );
        trace!(
            "`git reset --hard origin/{branch_name}`: {:?}",
            Command::new("git")
                .current_dir(&self.repo_directory)
                .args(["reset", "--hard", &format!("origin/{branch_name}")])
                .output()
                .expect("error running git command")
        );
    }
}
