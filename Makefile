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

# 🎼 Maestro Integration - Autonomous Development Orchestration
check-maestro: ## Check if Maestro is installed
	@command -v maestro >/dev/null 2>&1 || \
		(echo "❌ Maestro not found. Install with: pip install maestro-dev" && exit 1)
	@echo "✅ Maestro is available"

maestro-init: check-maestro ## Initialize Maestro in this project
	@echo "🎼 Initializing Maestro autonomous development..."
	maestro init
	@echo "✅ Maestro initialized for DBFast"

maestro-status: check-maestro ## Show Maestro orchestration status
	maestro status

# Single TDD Phase Orchestration
orchestrate-red: check-maestro ## Autonomous RED phase (usage: make orchestrate-red GOAL="user validation")
	@if [ -z "$(GOAL)" ]; then \
		echo "Usage: make orchestrate-red GOAL=\"implement user validation\""; \
		echo "Example: make orchestrate-red GOAL=\"add email validation tests\""; \
		exit 1; \
	fi
	@echo "🔴 Starting autonomous RED phase..."
	maestro orchestrate --phase red --goal "$(GOAL)"

orchestrate-green: check-maestro ## Autonomous GREEN phase (usage: make orchestrate-green GOAL="user validation")
	@if [ -z "$(GOAL)" ]; then \
		echo "Usage: make orchestrate-green GOAL=\"implement user validation\""; \
		echo "Example: make orchestrate-green GOAL=\"implement email validation\""; \
		exit 1; \
	fi
	@echo "🟢 Starting autonomous GREEN phase..."
	maestro orchestrate --phase green --goal "$(GOAL)"

orchestrate-refactor: check-maestro ## Autonomous REFACTOR phase (usage: make orchestrate-refactor GOAL="user validation")
	@if [ -z "$(GOAL)" ]; then \
		echo "Usage: make orchestrate-refactor GOAL=\"user validation cleanup\""; \
		echo "Example: make orchestrate-refactor GOAL=\"refactor email validation\""; \
		exit 1; \
	fi
	@echo "🔵 Starting autonomous REFACTOR phase..."
	maestro orchestrate --phase refactor --goal "$(GOAL)"

# Multi-Phase Campaign Orchestration
orchestrate-campaign: check-maestro ## Autonomous multi-phase development (usage: make orchestrate-campaign PHASES="2A,2B,2C" GOAL="auth system")
	@if [ -z "$(PHASES)" ] || [ -z "$(GOAL)" ]; then \
		echo "Usage: make orchestrate-campaign PHASES=\"2A,2B,2C\" GOAL=\"user authentication system\""; \
		echo "Example: make orchestrate-campaign PHASES=\"validation,creation,persistence\" GOAL=\"user management\""; \
		exit 1; \
	fi
	@echo "🎼 Starting autonomous development campaign..."
	maestro conduct --phases "$(PHASES)" --goal "$(GOAL)"

# Full Autonomous Development
orchestrate-symphony: check-maestro ## Full autonomous development from spec (usage: make orchestrate-symphony SPEC="requirements.md")
	@if [ -z "$(SPEC)" ]; then \
		echo "Usage: make orchestrate-symphony SPEC=\"requirements.md\" [TARGET=MVP]"; \
		echo "Example: make orchestrate-symphony SPEC=\"./docs/user-stories.md\" TARGET=\"Feature\""; \
		exit 1; \
	fi
	@if [ ! -f "$(SPEC)" ]; then \
		echo "❌ Specification file not found: $(SPEC)"; \
		exit 1; \
	fi
	@echo "🎼 Starting autonomous development symphony..."
	maestro symphony --spec "$(SPEC)" --target "$(or $(TARGET),MVP)"

# Convenient aliases
auto-red: orchestrate-red ## Alias for orchestrate-red
auto-green: orchestrate-green ## Alias for orchestrate-green
auto-refactor: orchestrate-refactor ## Alias for orchestrate-refactor
auto-campaign: orchestrate-campaign ## Alias for orchestrate-campaign
auto-symphony: orchestrate-symphony ## Alias for orchestrate-symphony
