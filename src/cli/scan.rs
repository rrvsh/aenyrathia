use clap::Args;
use ignore::Walk;
use std::path::Path;
use titlecase::titlecase;

#[derive(Args)]
pub struct ScanArgs {
    // The path to scan
    path: String,
}

impl ScanArgs {
    pub fn run(&self) {
        for entry in Walk::new(&self.path)
            .flatten()
            .filter(|x| !x.path().is_dir())
        {
            let mut components = entry.path().components();
            components.next();
            let pretty_components = components
                .map(|x| {
                    titlecase(
                        &Path::new(x.as_os_str())
                            .file_stem()
                            .expect("No file name found.")
                            .to_str()
                            .expect("Not valid Unicode.")
                            .replace('-', " "),
                    )
                })
                .collect::<Vec<_>>()
                .join("/");
            println!("{pretty_components:?}");
        }
    }
}
