# Zero2Prod Makefile
# Common development tasks and automation

.PHONY: help setup dev test build check clean fmt lint doc run watch coverage

# Default target
.DEFAULT_GOAL := help

# Color output
BLUE := \033[0;34m
GREEN := \033[0;32m
NC := \033[0m

help: ## Show this help message
	@echo '$(BLUE)Zero2Prod - Development Tasks$(NC)'
	@echo ''
	@echo '$(GREEN)Core Commands:$(NC)'
	@echo '  make setup              - Complete project setup (init.sh)'
	@echo '  make dev                - Start development environment'
	@echo '  make test               - Run all tests'
	@echo '  make build              - Build release binary'
	@echo ''
	@echo '$(GREEN)Development:$(NC)'
	@echo '  make run                - Run application'
	@echo '  make check              - Quick compile check'
	@echo '  make fmt                - Format code'
	@echo '  make lint               - Run clippy linter'
	@echo '  make doc                - Generate documentation'
	@echo '  make watch              - Watch and rebuild on changes'
	@echo ''
	@echo '$(GREEN)Database:$(NC)'
	@echo '  make migrate            - Run database migrations'
	@echo '  make migrate-new        - Create new migration'
	@echo '  make db-shell           - Open database shell'
	@echo ''
	@echo '$(GREEN)Maintenance:$(NC)'
	@echo '  make clean              - Clean build artifacts'
	@echo '  make coverage           - Generate code coverage report'
	@echo '  make deps               - Check for outdated dependencies'
	@echo '  make audit              - Check for security vulnerabilities'

# Setup Commands
setup: ## Complete project setup (runs init.sh)
	@bash init.sh

dev: ## Start development environment (run app with logging)
	@echo '$(GREEN)Starting development environment...$(NC)'
	@RUST_LOG=debug cargo run

# Test Commands
test: ## Run all tests
	@echo '$(GREEN)Running all tests...$(NC)'
	@cargo test

test-lib: ## Run unit tests only
	@echo '$(GREEN)Running unit tests...$(NC)'
	@cargo test --lib

test-integration: ## Run integration tests only
	@echo '$(GREEN)Running integration tests...$(NC)'
	@cargo test --test '*'

test-watch: ## Watch and re-run tests on file changes
	@echo '$(GREEN)Watching for changes and running tests...$(NC)'
	@cargo watch -x test

test-verbose: ## Run tests with output
	@echo '$(GREEN)Running tests with verbose output...$(NC)'
	@cargo test -- --nocapture --test-threads=1

# Build Commands
build: ## Build release binary
	@echo '$(GREEN)Building release binary...$(NC)'
	@cargo build --release

build-debug: ## Build debug binary
	@echo '$(GREEN)Building debug binary...$(NC)'
	@cargo build

check: ## Quick compile check (no build)
	@echo '$(GREEN)Checking code...$(NC)'
	@cargo check

# Run Commands
run: ## Run application
	@echo '$(GREEN)Running application...$(NC)'
	@cargo run

run-release: ## Run release binary
	@echo '$(GREEN)Running release binary...$(NC)'
	@cargo run --release

watch: ## Watch and rebuild on changes
	@echo '$(GREEN)Watching for changes...$(NC)'
	@cargo watch -x build

# Code Quality Commands
fmt: ## Format code with rustfmt
	@echo '$(GREEN)Formatting code...$(NC)'
	@cargo fmt

fmt-check: ## Check code formatting
	@echo '$(GREEN)Checking code format...$(NC)'
	@cargo fmt -- --check

lint: ## Run clippy linter
	@echo '$(GREEN)Running clippy linter...$(NC)'
	@cargo clippy --all-targets --all-features -- -D warnings

doc: ## Generate and open documentation
	@echo '$(GREEN)Generating documentation...$(NC)'
	@cargo doc --no-deps --open

# Database Commands
migrate: ## Run database migrations
	@echo '$(GREEN)Running migrations...$(NC)'
	@sqlx migrate run

migrate-add: ## Create new migration (usage: make migrate-add NAME=migration_name)
	@echo '$(GREEN)Creating migration: $(NAME)$(NC)'
	@sqlx migrate add -r $(NAME)

db-shell: ## Open PostgreSQL shell
	@psql -U postgres -d zero2prod

db-reset: ## Reset database (recreate from scratch)
	@echo '$(BLUE)⚠️  This will drop and recreate the database!$(NC)'
	@read -p "Are you sure? (y/N) " -n 1 -r; \
	if [[ $$REPLY =~ ^[Yy]$$ ]]; then \
		psql -U postgres -c "DROP DATABASE IF EXISTS zero2prod;" && \
		psql -U postgres -c "CREATE DATABASE zero2prod;" && \
		sqlx migrate run && \
		echo '$(GREEN)Database reset complete$(NC)'; \
	fi

# Analysis Commands
coverage: ## Generate code coverage report (requires tarpaulin)
	@echo '$(GREEN)Generating coverage report...$(NC)'
	@cargo tarpaulin --out Html

deps: ## Check for outdated dependencies
	@echo '$(GREEN)Checking for outdated dependencies...$(NC)'
	@cargo outdated

audit: ## Check for security vulnerabilities
	@echo '$(GREEN)Checking for vulnerabilities...$(NC)'
	@cargo audit

# Maintenance Commands
clean: ## Clean build artifacts and temporary files
	@echo '$(GREEN)Cleaning build artifacts...$(NC)'
	@cargo clean
	@rm -rf target/
	@echo '$(GREEN)Clean complete$(NC)'

distclean: ## Deep clean (removes Cargo.lock and target)
	@echo '$(BLUE)⚠️  Removing all build artifacts$(NC)'
	@cargo clean
	@rm -f Cargo.lock
	@echo '$(GREEN)Deep clean complete$(NC)'

# Development Workflow
pre-commit: fmt lint test ## Run pre-commit checks (fmt, lint, test)
	@echo '$(GREEN)Pre-commit checks passed!$(NC)'

ci: check fmt-check lint test doc ## Run CI checks (check, fmt-check, lint, test, doc)
	@echo '$(GREEN)CI checks passed!$(NC)'

# Git Commands
git-status: ## Show git status
	@git status

git-log: ## Show git log (last 10 commits)
	@git log --oneline -10

git-clean: ## Clean git working directory
	@git clean -fd

# Info Commands
info: ## Display project information
	@echo '$(BLUE)Zero2Prod Project Information$(NC)'
	@echo ''
	@echo 'Rust Version:    '$$(rustc --version)
	@echo 'Cargo Version:   '$$(cargo --version)
	@echo 'Package:         '$$(grep "^name" Cargo.toml | head -1 | cut -d'"' -f2)
	@echo 'Version:         '$$(grep "^version" Cargo.toml | head -1 | cut -d'"' -f2)
	@echo ''
	@cargo tree --depth 1

# Install tools
install-tools: ## Install development tools (cargo-watch, cargo-tarpaulin, etc)
	@echo '$(GREEN)Installing development tools...$(NC)'
	@cargo install cargo-watch || true
	@cargo install cargo-tarpaulin || true
	@cargo install cargo-outdated || true
	@cargo install cargo-audit || true
	@echo '$(GREEN)Tools installed$(NC)'

# Quick start
quickstart: ## Quick start (check, test, build, run)
	@$(MAKE) check
	@$(MAKE) test
	@$(MAKE) build
	@echo '$(GREEN)Ready to run! Try: make run$(NC)'

.PHONY: pre-commit ci install-tools quickstart info git-clean git-log git-status
.PHONY: distclean coverage audit deps clean db-reset db-shell migrate-add migrate
.PHONY: doc lint fmt fmt-check watch run-release run build-debug build check
.PHONY: test-verbose test-watch test-integration test-lib test dev setup help
