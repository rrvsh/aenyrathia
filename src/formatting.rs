use titlecase::titlecase;

#[must_use]
pub fn deslug(input: &str) -> String {
    titlecase(&input.replace('-', " "))
}
