.DEFAULT_GOAL := help
SHELL := /bin/sh
RUST_TOOLCHAIN ?= 1.98.0
CARGO ?= cargo
PYTHON ?= python3
GIT ?= git
CRATE ?=
TRACKED_FILES ?=

.PHONY: help info check-tools build build-debug build-release test test-default test-no-default test-all-features fmt fmt-check lint docs check coverage clean tracked-files planning planning-worktree publish publish-dry-run publish-one test-tools

help: ## Show available targets
	@awk 'BEGIN {FS = ":.*## "} /^[a-zA-Z0-9_-]+:.*## / {printf "%-22s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

info: ## Show workspace and selected toolchain
	@printf 'workspace: %s\ntoolchain: %s\n' "$(CURDIR)" "$(RUST_TOOLCHAIN)"

check-tools: ## Check required local build tools (coverage is optional)
	@command -v "$(CARGO)" >/dev/null || { echo "missing cargo: $(CARGO)" >&2; exit 1; }
	@$(CARGO) +$(RUST_TOOLCHAIN) --version >/dev/null
	@$(CARGO) +$(RUST_TOOLCHAIN) fmt --version >/dev/null
	@$(CARGO) +$(RUST_TOOLCHAIN) clippy --version >/dev/null
	@command -v "$(GIT)" >/dev/null || { echo "missing git: $(GIT)" >&2; exit 1; }
	@command -v "$(PYTHON)" >/dev/null || { echo "missing Python: $(PYTHON)" >&2; exit 1; }
	@echo "required tools available; coverage tooling is optional"

build: build-debug ## Build the workspace in debug mode
build-debug: ## Build workspace debug artifacts
	$(CARGO) +$(RUST_TOOLCHAIN) build --workspace --locked

build-release: ## Build workspace release artifacts
	$(CARGO) +$(RUST_TOOLCHAIN) build --workspace --release --locked

test: test-default ## Run the default-feature test suite
test-default: ## Run tests with default features
	$(CARGO) +$(RUST_TOOLCHAIN) test --workspace --locked

test-no-default: ## Run isolated search-crate tests without default features
	$(CARGO) +$(RUST_TOOLCHAIN) test --locked -p vlajmaji-search --no-default-features

test-all-features: ## Run tests with all features
	$(CARGO) +$(RUST_TOOLCHAIN) test --workspace --all-features --locked

fmt: ## Format workspace Rust sources
	$(CARGO) +$(RUST_TOOLCHAIN) fmt --all

fmt-check: ## Check Rust formatting without changes
	$(CARGO) +$(RUST_TOOLCHAIN) fmt --all -- --check

lint: ## Run Clippy for all targets with warnings denied
	$(CARGO) +$(RUST_TOOLCHAIN) clippy --workspace --all-targets --locked -- -D warnings

docs: ## Build workspace documentation
	$(CARGO) +$(RUST_TOOLCHAIN) doc --workspace --locked --no-deps

test-tools: ## Run local Make/planning helper tests
	PYTHONDONTWRITEBYTECODE=1 $(PYTHON) -m unittest discover -s tests -p 'test_build_tools.py' -v

check: check-tools fmt-check lint test-default docs test-tools ## Run standard local validation gates

coverage: ## Run optional coverage tooling when installed
	@command -v cargo-llvm-cov >/dev/null || { echo "coverage unavailable: install cargo-llvm-cov separately" >&2; exit 1; }
	$(CARGO) +$(RUST_TOOLCHAIN) llvm-cov --workspace --locked

clean: ## Remove Cargo build artifacts only
	$(CARGO) +$(RUST_TOOLCHAIN) clean

tracked-files: ## List tracked files, or write them to TRACKED_FILES when set
	@if [ -n "$(TRACKED_FILES)" ]; then $(GIT) ls-files > "$(TRACKED_FILES)"; else $(GIT) ls-files; fi

planning: ## Create or attach the safe planning worktree
	$(PYTHON) scripts/planning-worktree.py --remote origin --branch planning --path .worktrees/planning

planning-worktree: planning ## Alias for planning

publish: ## Publish workspace packages to crates.io (not run by validation)
	$(CARGO) +$(RUST_TOOLCHAIN) publish --workspace --registry crates-io --locked

publish-dry-run: ## Ask Cargo to validate workspace publishing (not run by validation)
	$(CARGO) +$(RUST_TOOLCHAIN) publish --workspace --registry crates-io --dry-run --locked

publish-one: ## Publish CRATE to crates.io (requires CRATE=package-name)
	@test -n "$(CRATE)" || { echo "set CRATE to a workspace package name" >&2; exit 2; }
	$(CARGO) +$(RUST_TOOLCHAIN) publish -p "$(CRATE)" --registry crates-io --locked
