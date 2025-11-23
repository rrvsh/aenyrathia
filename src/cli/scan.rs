use clap::Args;
use ignore::Walk;

#[derive(Args)]
pub struct ScanArgs {
    // The path to scan
    path: String,
}

impl ScanArgs {
    pub fn run(&self) {
        for entry in Walk::new(&self.path) {
            println!("{}", entry.unwrap().path().display());
        }
    }
}
