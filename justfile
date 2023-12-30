# List all recipes
default:
	just --list --unsorted

# Run all the tests
test:
	cargo test --workspace --release --locked

# Pre-check before publishing to crate
check:
	cargo clean
	just tests
	cargo fmt --all -- --check
	cargo clippy -- -D warnings
	cargo check

# compile
compile:
	cargo test --workspace --release --no-run --locked

# Lint check
lint:
	cargo clippy --release --workspace --locked --tests -- -Dwarnings
	cargo fmt --all --check

# Run the binary
run:
	cargo run --bin noaa info --station-id VOBL
