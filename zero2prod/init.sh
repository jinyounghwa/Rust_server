#!/bin/bash

# Zero2Prod - Complete Project Setup Script
# This script initializes the entire development environment
# Usage: bash init.sh [options]

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'  # No Color

# Configuration
POSTGRES_USER="${POSTGRES_USER:-postgres}"
POSTGRES_PASSWORD="${POSTGRES_PASSWORD:-password}"
POSTGRES_HOST="${POSTGRES_HOST:-localhost}"
POSTGRES_PORT="${POSTGRES_PORT:-5432}"
DATABASE_NAME="zero2prod"

# Functions
print_header() {
  echo -e "\n${BLUE}═══════════════════════════════════════${NC}"
  echo -e "${BLUE}$1${NC}"
  echo -e "${BLUE}═══════════════════════════════════════${NC}\n"
}

print_success() {
  echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
  echo -e "${RED}✗ $1${NC}"
}

print_warning() {
  echo -e "${YELLOW}⚠ $1${NC}"
}

print_info() {
  echo -e "${BLUE}ℹ $1${NC}"
}

# Check if command exists
command_exists() {
  command -v "$1" >/dev/null 2>&1
}

# Step 1: Check prerequisites
check_prerequisites() {
  print_header "Checking Prerequisites"

  if ! command_exists cargo; then
    print_error "Rust/Cargo not found. Please install Rust:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
  fi
  print_success "Rust/Cargo is installed"

  if ! command_exists psql; then
    print_warning "PostgreSQL not found. Please install PostgreSQL:"
    echo "  Windows: https://www.postgresql.org/download/windows/"
    echo "  macOS: brew install postgresql"
    echo "  Linux: sudo apt-get install postgresql"
    exit 1
  fi
  print_success "PostgreSQL is installed"

  if ! command_exists sqlx; then
    print_info "Installing sqlx-cli..."
    cargo install sqlx-cli --no-default-features --features postgres
    print_success "sqlx-cli installed"
  else
    print_success "sqlx-cli is installed"
  fi
}

# Step 2: Verify Rust installation
verify_rust() {
  print_header "Verifying Rust Installation"

  RUST_VERSION=$(rustc --version)
  print_success "Rust version: $RUST_VERSION"

  CARGO_VERSION=$(cargo --version)
  print_success "Cargo version: $CARGO_VERSION"
}

# Step 3: Create/verify database
setup_database() {
  print_header "Setting Up PostgreSQL Database"

  print_info "Attempting to connect to PostgreSQL at $POSTGRES_HOST:$POSTGRES_PORT..."

  # Try to create database
  if psql -U "$POSTGRES_USER" -h "$POSTGRES_HOST" -p "$POSTGRES_PORT" -tc \
    "SELECT 1 FROM pg_database WHERE datname = '$DATABASE_NAME'" | grep -q 1; then
    print_warning "Database '$DATABASE_NAME' already exists"
  else
    print_info "Creating database '$DATABASE_NAME'..."
    psql -U "$POSTGRES_USER" -h "$POSTGRES_HOST" -p "$POSTGRES_PORT" \
      -c "CREATE DATABASE $DATABASE_NAME;"
    print_success "Database created"
  fi
}

# Step 4: Set up environment file
setup_env() {
  print_header "Setting Up Environment"

  if [ ! -f .env ]; then
    print_info "Creating .env file..."
    cat > .env << EOF
# Zero2Prod Environment Configuration
DATABASE_URL=postgres://$POSTGRES_USER:$POSTGRES_PASSWORD@$POSTGRES_HOST:$POSTGRES_PORT/$DATABASE_NAME
RUST_LOG=info
EMAIL_CLIENT_URL=http://localhost:5001
EOF
    print_success ".env file created"
  else
    print_warning ".env file already exists (skipping)"
  fi
}

# Step 5: Run database migrations
run_migrations() {
  print_header "Running Database Migrations"

  export DATABASE_URL="postgres://$POSTGRES_USER:$POSTGRES_PASSWORD@$POSTGRES_HOST:$POSTGRES_PORT/$DATABASE_NAME"

  if [ -d "migrations" ]; then
    print_info "Running sqlx migrations..."
    sqlx migrate run
    print_success "Migrations completed"
  else
    print_warning "No migrations directory found"
  fi
}

# Step 6: Install Rust dependencies
install_dependencies() {
  print_header "Installing Rust Dependencies"

  print_info "Running 'cargo fetch' to download dependencies..."
  cargo fetch
  print_success "Dependencies downloaded"

  print_info "Running 'cargo check' to verify build..."
  cargo check
  print_success "Build check passed"
}

# Step 7: Run tests
run_tests() {
  print_header "Running Tests"

  print_info "Executing test suite..."
  cargo test --lib

  if [ $? -eq 0 ]; then
    print_success "All tests passed"
  else
    print_error "Some tests failed"
    exit 1
  fi
}

# Step 8: Display setup summary
display_summary() {
  print_header "Setup Complete! ✓"

  echo -e "${GREEN}Zero2Prod is ready for development${NC}\n"

  echo "Quick Start:"
  echo -e "  ${YELLOW}1. Start the application:${NC}"
  echo -e "     cargo run\n"

  echo -e "  ${YELLOW}2. In another terminal, run tests:${NC}"
  echo -e "     cargo test\n"

  echo "Useful Commands:"
  echo -e "  ${YELLOW}Build:${NC}          cargo build --release"
  echo -e "  ${YELLOW}Test:${NC}           cargo test"
  echo -e "  ${YELLOW}Lint:${NC}           cargo clippy"
  echo -e "  ${YELLOW}Format:${NC}         cargo fmt"
  echo -e "  ${YELLOW}Documentation:${NC}  cargo doc --open\n"

  echo "Database:"
  echo -e "  ${YELLOW}Connection:${NC}     postgres://$POSTGRES_USER@$POSTGRES_HOST:$POSTGRES_PORT/$DATABASE_NAME"
  echo -e "  ${YELLOW}Query DB:${NC}       psql -U $POSTGRES_USER -d $DATABASE_NAME\n"

  echo "API Endpoints (when running):"
  echo -e "  ${YELLOW}Health Check:${NC}   http://localhost:8002/health_check"
  echo -e "  ${YELLOW}Subscribe:${NC}      POST http://localhost:8002/subscriptions"
  echo -e "  ${YELLOW}Confirm:${NC}        GET http://localhost:8002/subscriptions/confirm\n"

  echo "Documentation:"
  echo -e "  ${YELLOW}Quick Start:${NC}     docs/QUICK_START.md"
  echo -e "  ${YELLOW}Setup Guide:${NC}     docs/SETUP_GUIDE.md"
  echo -e "  ${YELLOW}Security:${NC}        docs/SECURITY.md\n"
}

# Main execution
main() {
  print_header "Zero2Prod Setup Initialization"
  print_info "Setting up development environment...\n"

  check_prerequisites
  verify_rust
  setup_database
  setup_env
  run_migrations
  install_dependencies
  run_tests
  display_summary
}

# Run main function
main "$@"
