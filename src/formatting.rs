use std::path::Path;
use titlecase::titlecase;

#[must_use]
pub fn deslug(input: &str) -> String {
    titlecase(&input.replace('-', " "))
}

#[allow(clippy::missing_panics_doc)]
#[must_use]
pub fn normalise_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .expect("")
        .components()
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
        .join("/")
}
