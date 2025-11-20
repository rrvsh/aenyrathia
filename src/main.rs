use std::{fs,io};

fn main() -> io::Result<()> {
    let path = "docs";
    for entry in fs::read_dir(path)? {
        println!("{:?}", entry?)
    }
    Ok(())
}
