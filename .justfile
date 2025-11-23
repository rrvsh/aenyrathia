nice: format lint

lint:
  cargo clippy --fix --allow-dirty

format:
  cargo fmt

roll *ARGS:
  cargo run -- roll {{ARGS}}

run *ARGS:
  cargo run -- {{ARGS}}

build:
  cargo build

render *ARGS:
  cargo run -- render {{ARGS}}
