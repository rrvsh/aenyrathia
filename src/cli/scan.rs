use clap::Args;
use pinbreak::domain::filetree::Directory;

#[derive(Args)]
pub struct ScanArgs {
    // The path to scan
    path: String,
}

impl ScanArgs {
    pub fn run(&self) {
        Directory::from_path(&self.path).pretty_print();
    }
}
