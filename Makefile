.PHONY: help build test clean wasm docs

help: ## Show this help message
	@echo "Leptos-Sync Development Commands:"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

build: ## Build all crates
	cargo build --all

build-release: ## Build all crates in release mode
	cargo build --all --release

test: ## Run all tests
	cargo test --all

test-wasm: ## Run WASM tests
	wasm-pack test --node --headless

test-browser: ## Run browser tests
	wasm-pack test --chrome --headless

test-e2e: ## Run Playwright E2E tests
	playwright test

test-e2e-ui: ## Run Playwright E2E tests with UI
	playwright test --ui

test-e2e-headed: ## Run Playwright E2E tests in headed mode
	playwright test --headed

test-e2e-debug: ## Run Playwright E2E tests in debug mode
	playwright test --debug

test-all: test test-e2e ## Run all tests (unit + E2E)

clean: ## Clean build artifacts
	cargo clean
	rm -rf pkg/
	rm -rf target/

wasm: ## Build WASM packages
	./scripts/build-wasm.sh

docs: ## Generate documentation
	cargo doc --all --no-deps --open

check: ## Run all checks
	cargo check --all
	cargo clippy --all -- -D warnings
	cargo fmt --all -- --check

audit: ## Security audit
	cargo audit

coverage: ## Generate test coverage
	cargo tarpaulin --out html --output-dir coverage

install-deps: ## Install development dependencies
	cargo install wasm-pack
	cargo install wasm-opt
	cargo install cargo-audit
	cargo install cargo-tarpaulin
	cargo install cargo-deny
	cargo install cargo-flamegraph
	cargo install cargo-leptos

setup: install-deps ## Complete development setup
	rustup target add wasm32-unknown-unknown
	git config core.hooksPath .githooks
	chmod +x .githooks/*
