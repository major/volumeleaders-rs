.PHONY: audit check clippy clean coverage doc fmt test

check: fmt clippy test doc

fmt:
	@cargo fmt --all

clippy:
	@cargo clippy --workspace -- -D clippy::all

test:
	@cargo test --workspace

doc:
	@cargo doc --workspace --no-deps

coverage:
	@cargo llvm-cov --workspace --fail-under-lines 90

audit:
	@cargo audit

clean:
	@cargo clean
