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

keygen:
  mkdir -p ./secrets
  ssh-keyscan github.com > ./secrets/known_hosts
  ssh-keygen -f ./secrets/deploy_aenyrathia
  echo "Please add \`./secrets/deploy_aenyrathia.pub\` as a verified public key for the git remote."

run-docker:
  docker build . -f ./ops/Dockerfile -t aenyrathia:latest
  docker run --rm -p 3000:3000 \
    -v ./secrets/deploy_aenyrathia:/root/.ssh/id_ed25519:ro \
    -v ./secrets/known_hosts:/root/.ssh/known_hosts:ro \
    aenyrathia:latest

setup:
  docker-compose up -d

reset:
  docker-compose down -v
