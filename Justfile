nice: format lint

lint:
  cargo clippy --fix --allow-dirty

format:
  cargo fmt

run *ARGS:
  cargo run -- {{ARGS}}

build:
  cargo build

render *ARGS:
  cargo run -- render {{ARGS}}

serve *ARGS:
  cargo run -- serve {{ARGS}}
