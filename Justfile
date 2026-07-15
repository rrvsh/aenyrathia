nice: format lint

lint:
	cargo clippy --fix --allow-dirty

format:
	cargo fmt

run *ARGS:
	cargo run -- {{ARGS}}

build:
	cargo build

test:
	cargo test

package:
	nix build .#aenyrathia

keygen:
	mkdir -p ./secrets
	ssh-keyscan github.com > ./secrets/known_hosts
	ssh-keygen -f ./secrets/deploy_aenyrathia
	echo "Set GIT_SSH_KEY_PATH to ./secrets/deploy_aenyrathia for local testing."
	echo "Add ./secrets/deploy_aenyrathia.pub as a deploy key for the git remote."
