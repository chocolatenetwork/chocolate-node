run-tmp:
	SKIP_WASM_BUILD= cargo run -- --dev --tmp

run:
	SKIP_WASM_BUILD= cargo run -- --dev

toolchain:
	./scripts/init.sh

build:
	cargo build --release

check:
	SKIP_WASM_BUILD= cargo check --all --tests

test:
	SKIP_WASM_BUILD= cargo test --all

purge:
	SKIP_WASM_BUILD= cargo run -- purge-chain --dev -y

restart: purge run

init: toolchain build-full
