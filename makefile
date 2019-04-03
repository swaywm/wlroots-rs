default: unstable_static_build

unstable_static_build:
	cargo build --features "unstable static"

.PHONY: examples
examples:
	cargo build  --examples --features "unstable static"

clippy:
	cargo clippy --features "unstable static" --examples

build:
	cargo build

