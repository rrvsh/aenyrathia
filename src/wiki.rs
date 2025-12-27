use log::{debug, trace, warn};
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Checks if a file path is editable by checking if any of the following conditions are true:
/// 1. The file already exists and is not a directory
/// 2. The parent directory exists
fn path_editable<P: AsRef<std::path::Path>>(path: P) -> bool {
    let path = path.as_ref();
    path.is_file() || path.parent().is_some_and(std::path::Path::is_dir)
}

#[derive(Clone)]
pub struct Wiki {
    repo_directory: String,
}

impl Wiki {
    pub fn from_remote(git_remote: &str, tempdir: &TempDir) -> Self {
        trace!("Initialising wiki from git remote.");
        let repo_directory = tempdir
            .path()
            .to_str()
            .expect("Invalid UTF-8 in tempdir path!");
        trace!(
            "`git clone {git_remote} {repo_directory}`: {:?}",
            std::process::Command::new("git")
                .args(["clone", git_remote, repo_directory])
                .output()
                .expect("git command failed to start")
        );
        trace!(
            "`git config user.name aenyrathia.wiki`: {:?}",
            std::process::Command::new("git")
                .current_dir(repo_directory)
                .args(["config", "user.name", "aenyrathia.wiki"])
                .output()
                .expect("git command failed to start")
        );
        trace!(
            "`git config user.email git@aenyrathia.wiki`: {:?}",
            std::process::Command::new("git")
                .current_dir(repo_directory)
                .args(["config", "user.email", "git@aenyrathia.wiki"])
                .output()
                .expect("git command failed to start")
        );

        Self {
            repo_directory: repo_directory.to_string(),
        }
    }

    /// Resolves `{article_path}` to `wiki/{article_path}.md`.
    fn resolve_article_path(&self, article_path: &str) -> std::path::PathBuf {
        let ensured_article_path = if article_path.is_empty() {
            "index"
        } else {
            article_path
        };
        PathBuf::from(&self.repo_directory)
            .join("wiki")
            .join(ensured_article_path)
            .with_extension("md")
    }

    /// Switches the git repo to `branch_name`
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

    /// Rebases `branch_name`
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
            "`git rebase origin/prime`: {:?}",
            Command::new("git")
                .current_dir(&self.repo_directory)
                .args(["rebase", "origin/prime"])
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
            "`git push origin {branch_name}`: {:?}",
            Command::new("git")
                .current_dir(&self.repo_directory)
                .args(["push", "origin", branch_name])
                .output()
                .expect("error running git command")
        );
    }

    pub fn update_remote_branch_file_contents(
        self,
        article_path: &str,
        new_file_content: &str,
        branch_name: &str,
    ) -> Result<(), ()> {
        self.checkout_remote_branch(branch_name);
        let path = self.resolve_article_path(article_path);
        if path_editable(&path) {
            std::fs::write(&path, new_file_content).map_or_else(
                |e| {
                    warn!("Couldn't write to file {}: {}", path.display(), e);
                    Err(())
                },
                |()| {
                    trace!("Wrote to file {}", path.display());
                    self.update_and_sync(branch_name);
                    Ok(())
                },
            )
        } else {
            debug!("{} not editable.", path.display());
            Err(())
        }
    }

    pub fn get_remote_branch_file_contents(
        self,
        article_path: &str,
        branch_name: &str,
    ) -> Option<String> {
        self.checkout_remote_branch(branch_name);
        let path = self.resolve_article_path(article_path);
        std::fs::read_to_string(&path).map_or_else(
            |e| {
                warn!("Couldn't read {} to string: {}", path.display(), e);
                None
                //FIXME: if file_path is editable, edit_mode, and logged in, render empty file_content
            },
            Some,
        )
    }
}
