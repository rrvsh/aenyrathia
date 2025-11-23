use clap::Args;
use pinbreak::domain::filetree::pretty_print_dir;

#[derive(Args)]
pub struct ScanArgs {
    // The path to scan
    path: String,
}

impl ScanArgs {
    pub fn run(&self) {
        pretty_print_dir(&self.path);
    }
}
