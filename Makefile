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

# Phase orchestrator targets (following Phase 6 methodology)
phase-status: ## Show development phase progress
	@./scripts/phase-status.sh

check-all: quality ## Run comprehensive quality checks (alias for quality)

commit-tdd: ## Commit with --no-verify for TDD RED/GREEN phases
	@echo "🔴🟢 TDD commit - bypassing pre-commit hooks"
	@echo "⚠️  Remember to refactor and use commit-clean!"
	git commit --no-verify

commit-clean: ## Commit with full quality checks for REFACTOR phases
	@echo "🔍 Running full quality checks before commit..."
	@$(MAKE) check-all
	git commit
	@echo "✅ Clean commit completed"

commit-wip: ## Commit work-in-progress with --no-verify
	@echo "⚠️  WIP commit - bypassing pre-commit hooks"
	@echo "🚨 Remember to clean up before PR!"
	git commit --no-verify

prepare-pr: ## Prepare branch for pull request
	@echo "🔍 Preparing for pull request..."
	@echo "📊 Running comprehensive quality checks..."
	@$(MAKE) check-all
	@echo "🔍 Checking for --no-verify commits..."
	@if git log --grep="🔴\|🟢" --oneline dev..HEAD 2>/dev/null | head -5; then \
		echo "💡 Consider cleaning up TDD commits with: git rebase -i dev"; \
	else \
		echo "✅ No TDD commits found"; \
	fi
	@echo "✅ Ready for PR creation!"

# Phase management shortcuts
start-phase: ## Start a new phase (usage: make start-phase PHASE=2A)
	@if [ -z "$(PHASE)" ]; then \
		echo "Usage: make start-phase PHASE=2A"; \
		echo "Available phases: 2A, 2B, 2C, 2D, 3B, 4A, 4B"; \
		exit 1; \
	fi
	@./scripts/run-phase.sh $(PHASE)

complete-phase: ## Complete current phase (usage: make complete-phase PHASE=2A)
	@if [ -z "$(PHASE)" ]; then \
		echo "Usage: make complete-phase PHASE=2A"; \
		exit 1; \
	fi
	@./scripts/complete-phase.sh $(PHASE)

# 🎼✨ Vision-to-Code Transformation (Ultimate Maestro)
vision-to-code: check-maestro ## Transform high-level vision into production code (usage: make vision-to-code VISION="description")
	@if [ -z "$(VISION)" ]; then \
		echo "Usage: make vision-to-code VISION=\"A REST API for user management with JWT authentication\""; \
		echo "Examples:"; \
		echo "  make vision-to-code VISION=\"Add password reset functionality via email\""; \
		echo "  make vision-to-code VISION=\"Real-time chat system with WebSocket support\""; \
		echo "  make vision-to-code VISION=\"GraphQL API with subscription support\""; \
		exit 1; \
	fi
	@echo "🎼✨ Starting vision-to-code transformation..."
	@echo "💡 Vision: $(VISION)"
	maestro create --vision "$(VISION)" --target "$(or $(TARGET),production)"

add-feature: check-maestro ## Add new feature to existing codebase (usage: make add-feature FEATURE="description")
	@if [ -z "$(FEATURE)" ]; then \
		echo "Usage: make add-feature FEATURE=\"Password reset via email\""; \
		echo "Examples:"; \
		echo "  make add-feature FEATURE=\"Add OAuth2 social login support\""; \
		echo "  make add-feature FEATURE=\"Implement rate limiting middleware\""; \
		echo "  make add-feature FEATURE=\"Add real-time notifications\""; \
		exit 1; \
	fi
	@echo "🎼+ Adding feature to existing codebase..."
	@echo "✨ Feature: $(FEATURE)"
	maestro add-feature --feature "$(FEATURE)"

# Enhanced aliases with vision support
create-app: vision-to-code ## Alias for vision-to-code
think-and-build: vision-to-code ## Alias for vision-to-code  
maestro-create: vision-to-code ## Alias for vision-to-code
