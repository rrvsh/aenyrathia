use crate::formatting::normalise_path;
use ignore::WalkBuilder;
use std::path::Path;

#[derive(Debug)]
enum FileTree {
    FileNode(File),
    DirectoryNode(Directory),
}

#[derive(Debug)]
pub struct File {
    name: String,
}

#[derive(Debug)]
pub struct Directory {
    name: String,
    children: Vec<FileTree>,
}

impl Directory {
    pub fn pretty_print(&self) {
        if !self.name.is_empty() {
            println!("{}", self.name);
        }
        for child in &self.children {
            match child {
                FileTree::FileNode(file) => println!("{}", file.name),
                FileTree::DirectoryNode(directory) => directory.pretty_print(),
            }
        }
    }

    #[must_use]
    pub fn from_path(path: &str) -> Self {
        let root = Path::new(path);
        Self::recurse_path(root, root)
    }

    fn recurse_path(root: &Path, path: &Path) -> Self {
        let mut children: Vec<FileTree> = Vec::new();
        let entries = WalkBuilder::new(path).max_depth(Some(1)).build().flatten();
        for entry in entries {
            let entry_path = entry.path();
            if entry_path == path {
                continue;
            }
            if entry_path.is_dir() {
                children.push(FileTree::DirectoryNode(Self::recurse_path(
                    root, entry_path,
                )));
            } else {
                children.push(FileTree::FileNode(File {
                    name: normalise_path(root, entry_path),
                }));
            }
        }
        Self {
            name: normalise_path(root, path),
            children,
        }
    }
}
