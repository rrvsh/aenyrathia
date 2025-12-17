use env_logger::Builder;
use env_logger::Env;
use std::io;

mod cli;

fn main() -> io::Result<()> {
    let env = Env::new().filter("PB_LOG");
    let mut builder = Builder::from_env(env);
    builder.init();

    cli::run()
}
