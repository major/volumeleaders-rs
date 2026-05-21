.PHONY: audit check clippy clean coverage doc fmt patch-coverage test

PATCH_COVERAGE_BASE ?= main
PATCH_COVERAGE_FAIL_UNDER ?= 100
DIFF_COVER ?= diff-cover

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

patch-coverage:
	@cargo llvm-cov --workspace --fail-under-lines 90 --lcov --output-path lcov.info
	@$(DIFF_COVER) lcov.info --compare-branch=$(PATCH_COVERAGE_BASE) --fail-under=$(PATCH_COVERAGE_FAIL_UNDER)

audit:
	@cargo audit

clean:
	@cargo clean
	@rm -f lcov.info
