# Makefile for dbfast
# Generated on 2025-09-10

.PHONY: help install check test fmt clippy audit deny doc bench coverage clean release

# Default target
help: ## Show this help message
	@echo "Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

# Development setup
install: ## Install required tools
	@echo "Installing required tools..."
	cargo install cargo-nextest cargo-audit cargo-deny cargo-tarpaulin cargo-outdated
	pip install pre-commit
	pre-commit install

# Code quality checks
check: ## Run cargo check
	cargo check --all-targets --all-features

test: ## Run tests with nextest
	cargo nextest run --all-features

test-doc: ## Run documentation tests
	cargo test --doc --all-features

fmt: ## Format code with rustfmt
	cargo fmt --all

fmt-check: ## Check code formatting
	cargo fmt --all -- --check

clippy: ## Run clippy lints
	cargo clippy --all-targets --all-features -- -D warnings

# Security and dependency management
audit: ## Audit dependencies for security vulnerabilities
	cargo audit

deny: ## Check dependencies with cargo-deny
	cargo deny check

outdated: ## Check for outdated dependencies
	cargo outdated

# Documentation
doc: ## Generate and open documentation
	cargo doc --no-deps --document-private-items --all-features --open

doc-check: ## Check documentation
	cargo doc --no-deps --document-private-items --all-features
	RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features

# Performance
bench: ## Run benchmarks
	cargo bench

# Coverage
coverage: ## Generate test coverage report
	cargo tarpaulin --verbose --all-features --workspace --timeout 120 --out html

# Quality gate - run all checks
quality: check test fmt-check clippy audit deny doc-check ## Run all quality checks

# Clean up
clean: ## Clean build artifacts
	cargo clean
	rm -rf target/
	rm -rf tarpaulin-report.html

# Release
release-dry: ## Dry run release process
	cargo publish --dry-run

release: quality ## Build and publish release (requires CARGO_REGISTRY_TOKEN)
	cargo publish

# Pre-commit
pre-commit: ## Run pre-commit hooks
	pre-commit run --all-files

# Development workflow
dev: fmt clippy test ## Quick development workflow

# CI simulation
ci: quality coverage ## Simulate CI pipeline locally