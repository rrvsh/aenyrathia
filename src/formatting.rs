/// Replaces all occurences of `\r\n` and `\r` with `\n`.
pub fn normalise_newlines(input: &str) -> String {
    input.replace("\r\n", "\n").replace('\r', "\n")
}
