# Makefile for the Vlajmaji Project
.DEFAULT_GOAL := help
SHELL := /bin/sh

# ANSI colour palette shared with Oxurack.
BLUE := \033[1;34m
GREEN := \033[1;32m
YELLOW := \033[1;33m
RED := \033[1;31m
CYAN := \033[1;36m
RESET := \033[0m

PROJECT_NAME := Vlajmaji
BINARIES := vlajmaji
RUST_TOOLCHAIN ?= 1.98.0
CARGO ?= cargo
PYTHON ?= python3
GIT ?= git
CRATE ?=
TRACKED_FILES ?=
MODE ?= debug
INSTALL_DIR ?= ./bin
CARGO_TARGET_DIR ?= target
export CARGO_TARGET_DIR
TARGET_DIR = $(CARGO_TARGET_DIR)/$(MODE)
GIT_COMMIT = $(shell $(GIT) rev-parse --short HEAD 2>/dev/null || echo unknown)
GIT_BRANCH = $(shell $(GIT) rev-parse --abbrev-ref HEAD 2>/dev/null || echo unknown)
BUILD_TIME = $(shell date -u '+%Y-%m-%dT%H:%M:%SZ')

# Escape spaces for Make's directory target; recipes use the original path.
_empty :=
_space := $(_empty) $(_empty)
INSTALL_DIR_TARGET := $(subst $(_space),\ ,$(INSTALL_DIR))

.PHONY: help info check-tools build build-debug build-release install test test-default test-no-default test-all-features fmt format fmt-check lint docs check coverage clean tracked-files planning planning-worktree publish publish-dry-run publish-one test-tools

help:
	@printf '\n%b╔══════════════════════════════════════════════════════════╗%b\n' '$(CYAN)' '$(RESET)'
	@printf '%b║%b %b%-56s%b %b║%b\n' '$(CYAN)' '$(RESET)' '$(BLUE)' '$(PROJECT_NAME) Build System' '$(RESET)' '$(CYAN)' '$(RESET)'
	@printf '%b╚══════════════════════════════════════════════════════════╝%b\n\n' '$(CYAN)' '$(RESET)'
	@printf '%bBuilding:%b\n' '$(GREEN)' '$(RESET)'
	@printf '  %bmake build%b             - Build and install $(BINARIES) to $(INSTALL_DIR)\n' '$(YELLOW)' '$(RESET)'
	@printf '  %bmake build-release%b     - Build and install optimized release binaries\n' '$(YELLOW)' '$(RESET)'
	@printf '  %bmake install%b           - Copy existing binaries (MODE=debug or release)\n' '$(YELLOW)' '$(RESET)'
	@printf '  %bmake docs%b              - Generate API documentation\n\n' '$(YELLOW)' '$(RESET)'
	@printf '%bTesting & Quality:%b\n' '$(GREEN)' '$(RESET)'
	@printf '  %bmake test%b              - Run default-feature workspace tests\n' '$(YELLOW)' '$(RESET)'
	@printf '  %bmake test-no-default%b   - Run isolated search tests without default features\n' '$(YELLOW)' '$(RESET)'
	@printf '  %bmake test-all-features%b - Run all-feature workspace tests\n' '$(YELLOW)' '$(RESET)'
	@printf '  %bmake test-tools%b        - Run Make/planning helper tests\n' '$(YELLOW)' '$(RESET)'
	@printf '  %bmake lint%b              - Run Clippy with warnings denied\n' '$(YELLOW)' '$(RESET)'
	@printf '  %bmake format%b            - Format all Rust code\n' '$(YELLOW)' '$(RESET)'
	@printf '  %bmake fmt-check%b         - Check Rust formatting\n' '$(YELLOW)' '$(RESET)'
	@printf '  %bmake coverage%b          - Run optional coverage tooling\n' '$(YELLOW)' '$(RESET)'
	@printf '  %bmake check%b             - Tools + format + lint + tests + docs\n\n' '$(YELLOW)' '$(RESET)'
	@printf '%bCleaning:%b\n' '$(GREEN)' '$(RESET)'
	@printf '  %bmake clean%b             - Clean Cargo build artifacts\n\n' '$(YELLOW)' '$(RESET)'
	@printf '%bUtilities:%b\n' '$(GREEN)' '$(RESET)'
	@printf '  %bmake planning%b          - Fetch or create the planning worktree\n' '$(YELLOW)' '$(RESET)'
	@printf '  %bmake publish-dry-run%b   - Verify crates.io packaging without uploading\n' '$(YELLOW)' '$(RESET)'
	@printf '  %bmake publish%b           - Publish workspace crates to crates.io\n' '$(YELLOW)' '$(RESET)'
	@printf '  %bmake publish-one%b       - Publish one package (CRATE=package-name)\n' '$(YELLOW)' '$(RESET)'
	@printf '  %bmake tracked-files%b     - List files or write to TRACKED_FILES\n\n' '$(YELLOW)' '$(RESET)'
	@printf '%bInformation:%b\n' '$(GREEN)' '$(RESET)'
	@printf '  %bmake info%b              - Show build information\n' '$(YELLOW)' '$(RESET)'
	@printf '  %bmake check-tools%b       - Check required tools\n\n' '$(YELLOW)' '$(RESET)'
	@printf '%bCurrent status:%b Branch: %s | Commit: %s\n\n' '$(CYAN)' '$(RESET)' '$(GIT_BRANCH)' '$(GIT_COMMIT)'

info:
	@printf '\n%bBuild Information%b\n\n' '$(BLUE)' '$(RESET)'
	@printf '%bProject:%b\n  Name: %s\n  Mode: %s\n  Build Time: %s\n' '$(GREEN)' '$(RESET)' '$(PROJECT_NAME)' '$(MODE)' '$(BUILD_TIME)'
	@printf '%bPaths:%b\n  Workspace: %s\n  Target: %s\n  Install: %s\n' '$(GREEN)' '$(RESET)' '$(CURDIR)' '$(TARGET_DIR)' '$(INSTALL_DIR)'
	@printf '%bGit:%b\n  Branch: %s\n  Commit: %s\n' '$(GREEN)' '$(RESET)' '$(GIT_BRANCH)' '$(GIT_COMMIT)'
	@printf '%bTools:%b\n  Rust toolchain: %s\n' '$(GREEN)' '$(RESET)' '$(RUST_TOOLCHAIN)'
	@$(CARGO) +$(RUST_TOOLCHAIN) --version

check-tools:
	@printf '%bChecking for required tools...%b\n' '$(BLUE)' '$(RESET)'
	@command -v "$(CARGO)" >/dev/null || { printf '%b✗ cargo not found%b\n' '$(RED)' '$(RESET)' >&2; exit 1; }
	@$(CARGO) +$(RUST_TOOLCHAIN) --version >/dev/null
	@$(CARGO) +$(RUST_TOOLCHAIN) fmt --version >/dev/null
	@$(CARGO) +$(RUST_TOOLCHAIN) clippy --version >/dev/null
	@command -v "$(GIT)" >/dev/null || { printf '%b✗ git not found%b\n' '$(RED)' '$(RESET)' >&2; exit 1; }
	@command -v "$(PYTHON)" >/dev/null || { printf '%b✗ Python not found%b\n' '$(RED)' '$(RESET)' >&2; exit 1; }
	@printf '%b✓ Required tools available%b\n%b→ Coverage tooling is optional%b\n' '$(GREEN)' '$(RESET)' '$(YELLOW)' '$(RESET)'

build:
	@printf '%bBuilding $(PROJECT_NAME) in $(MODE) mode...%b\n' '$(BLUE)' '$(RESET)'
	@case "$(MODE)" in \
		debug) $(CARGO) +$(RUST_TOOLCHAIN) build --workspace --locked ;; \
		release) $(CARGO) +$(RUST_TOOLCHAIN) build --workspace --release --locked ;; \
		*) printf '%b✗ MODE must be debug or release%b\n' '$(RED)' '$(RESET)' >&2; exit 2 ;; \
	esac
	@$(MAKE) --no-print-directory install
	@printf '%b✓ Build complete%b\n' '$(GREEN)' '$(RESET)'

build-debug:
	@$(MAKE) --no-print-directory build MODE=debug

build-release:
	@$(MAKE) --no-print-directory build MODE=release

$(INSTALL_DIR_TARGET):
	@printf '%bCreating install directory...%b\n' '$(BLUE)' '$(RESET)'
	@mkdir -p "$(INSTALL_DIR)"
	@printf '%b✓ Directory ready: %s%b\n' '$(GREEN)' '$(INSTALL_DIR)' '$(RESET)'

install: $(INSTALL_DIR_TARGET)
	@case "$(MODE)" in debug|release) ;; *) printf '%b✗ MODE must be debug or release%b\n' '$(RED)' '$(RESET)' >&2; exit 2 ;; esac
	@printf '%b• Copying binaries to %s/%b\n' '$(CYAN)' '$(INSTALL_DIR)' '$(RESET)'
	@set -eu; for binary in $(BINARIES); do \
		cp "$(TARGET_DIR)/$$binary" "$(INSTALL_DIR)/$$binary"; \
		printf '  %b✓%b %s\n' '$(GREEN)' '$(RESET)' "$$binary"; \
	done
	@printf '%b→ Binaries available in %s/%b\n' '$(CYAN)' '$(INSTALL_DIR)' '$(RESET)'

test: test-default

test-default:
	@printf '%bRunning default-feature workspace tests...%b\n' '$(BLUE)' '$(RESET)'
	@$(CARGO) +$(RUST_TOOLCHAIN) test --workspace --locked
	@printf '%b✓ All tests passed%b\n' '$(GREEN)' '$(RESET)'

test-no-default:
	@printf '%bRunning isolated search tests without default features...%b\n' '$(BLUE)' '$(RESET)'
	@$(CARGO) +$(RUST_TOOLCHAIN) test --locked -p vlajmaji-search --no-default-features
	@printf '%b✓ No-default-feature tests passed%b\n' '$(GREEN)' '$(RESET)'

test-all-features:
	@printf '%bRunning all-feature workspace tests...%b\n' '$(BLUE)' '$(RESET)'
	@$(CARGO) +$(RUST_TOOLCHAIN) test --workspace --all-features --locked
	@printf '%b✓ All-feature tests passed%b\n' '$(GREEN)' '$(RESET)'

fmt:
	@printf '%bFormatting Rust code...%b\n' '$(BLUE)' '$(RESET)'
	@$(CARGO) +$(RUST_TOOLCHAIN) fmt --all
	@printf '%b✓ Code formatted%b\n' '$(GREEN)' '$(RESET)'

format: fmt

fmt-check:
	@printf '%bChecking code formatting...%b\n' '$(BLUE)' '$(RESET)'
	@$(CARGO) +$(RUST_TOOLCHAIN) fmt --all -- --check
	@printf '%b✓ Format check passed%b\n' '$(GREEN)' '$(RESET)'

lint:
	@printf '%bRunning Clippy...%b\n' '$(BLUE)' '$(RESET)'
	@$(CARGO) +$(RUST_TOOLCHAIN) clippy --workspace --all-targets --locked -- -D warnings
	@printf '%b✓ Clippy passed%b\n' '$(GREEN)' '$(RESET)'

docs:
	@printf '%bGenerating API documentation...%b\n' '$(BLUE)' '$(RESET)'
	@$(CARGO) +$(RUST_TOOLCHAIN) doc --workspace --locked --no-deps
	@printf '%b✓ Documentation generated%b\n' '$(GREEN)' '$(RESET)'

test-tools:
	@printf '%bRunning Make/planning helper tests...%b\n' '$(BLUE)' '$(RESET)'
	@PYTHONDONTWRITEBYTECODE=1 $(PYTHON) -m unittest discover -s tests -p 'test_build_tools.py' -v
	@printf '%b✓ Tooling tests passed%b\n' '$(GREEN)' '$(RESET)'

check: check-tools fmt-check lint test-default docs test-tools
	@printf '\n%b✓ All checks passed (tools + format + lint + tests + docs)%b\n\n' '$(GREEN)' '$(RESET)'

coverage:
	@printf '%bGenerating test coverage...%b\n' '$(BLUE)' '$(RESET)'
	@command -v cargo-llvm-cov >/dev/null || { printf '%b✗ Install cargo-llvm-cov separately%b\n' '$(RED)' '$(RESET)' >&2; exit 1; }
	@$(CARGO) +$(RUST_TOOLCHAIN) llvm-cov --workspace --locked
	@printf '%b✓ Coverage report generated%b\n' '$(GREEN)' '$(RESET)'

clean:
	@printf '%bCleaning Cargo build artifacts...%b\n' '$(BLUE)' '$(RESET)'
	@$(CARGO) +$(RUST_TOOLCHAIN) clean
	@printf '%b✓ Clean complete%b\n' '$(GREEN)' '$(RESET)'

tracked-files:
	@if [ -n "$(TRACKED_FILES)" ]; then $(GIT) ls-files > "$(TRACKED_FILES)"; else $(GIT) ls-files; fi

planning:
	@printf '%bSetting up the planning worktree...%b\n' '$(BLUE)' '$(RESET)'
	@$(PYTHON) scripts/planning-worktree.py --remote origin --branch planning --path .worktrees/planning
	@printf '%b✓ Planning worktree ready%b\n' '$(GREEN)' '$(RESET)'

planning-worktree: planning

publish:
	@printf '%bPublishing $(PROJECT_NAME) crates to crates.io...%b\n' '$(BLUE)' '$(RESET)'
	@$(CARGO) +$(RUST_TOOLCHAIN) publish --workspace --registry crates-io --locked
	@printf '%b✓ All crates published%b\n' '$(GREEN)' '$(RESET)'

publish-dry-run:
	@printf '%bDry run: publishing $(PROJECT_NAME) crates...%b\n' '$(BLUE)' '$(RESET)'
	@$(CARGO) +$(RUST_TOOLCHAIN) publish --workspace --registry crates-io --dry-run --locked
	@printf '%b✓ Publishing dry run passed%b\n' '$(GREEN)' '$(RESET)'

publish-one:
	@test -n "$(CRATE)" || { printf '%b✗ Set CRATE to a workspace package name%b\n' '$(RED)' '$(RESET)' >&2; exit 2; }
	@printf '%bPublishing %s...%b\n' '$(BLUE)' '$(CRATE)' '$(RESET)'
	@$(CARGO) +$(RUST_TOOLCHAIN) publish -p "$(CRATE)" --registry crates-io --locked
	@printf '%b✓ Published %s%b\n' '$(GREEN)' '$(CRATE)' '$(RESET)'
