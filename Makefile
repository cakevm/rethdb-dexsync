.PHONY: build run test clean fmt fmt-check deny-check clippy

build:
	cargo build --all

release:
	cargo build --release

run:
	cargo run

test:
	cargo test

clean:
	cargo clean

fmt:
	cargo fmt

fmt-check:
	cargo fmt --all --check

deny-check:
	cargo deny --all-features check

clippy:
	cargo clippy --all --all-features -- -D warnings