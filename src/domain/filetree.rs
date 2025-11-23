use crate::formatting::deslug;
use ignore::Walk;
use std::path::Path;

#[allow(clippy::missing_panics_doc)] // temporary until we refactor into a struct
pub fn pretty_print_dir(path: &str) {
    for entry in Walk::new(path).flatten().filter(|x| !x.path().is_dir()) {
        let mut components = entry.path().components();
        components.next();
        let pretty_components = components
            .map(|x| {
                deslug(
                    Path::new(x.as_os_str())
                        .file_stem()
                        .expect("No file name found.")
                        .to_str()
                        .expect("Not valid Unicode."),
                )
            })
            .collect::<Vec<_>>()
            .join("/");
        println!("{pretty_components:?}");
    }
}
