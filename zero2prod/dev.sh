#!/bin/bash

# Zero2Prod - Development Quick Start Script
# Starts the development environment with all necessary components
# Usage: bash dev.sh [options]

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

# Configuration
LOG_LEVEL="${RUST_LOG:-info}"
PORT="${PORT:-8002}"
DATABASE_URL="${DATABASE_URL:-postgres://postgres:password@localhost:5432/zero2prod}"

# Options
SKIP_CHECKS=false
NO_TESTS=false
WATCH=false

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --skip-checks) SKIP_CHECKS=true; shift ;;
    --no-tests) NO_TESTS=true; shift ;;
    --watch) WATCH=true; shift ;;
    --log-level) LOG_LEVEL="$2"; shift 2 ;;
    --port) PORT="$2"; shift 2 ;;
    --help) show_help; exit 0 ;;
    *) echo "Unknown option: $1"; exit 1 ;;
  esac
done

# Functions
print_header() {
  echo -e "\n${BLUE}╔════════════════════════════════════════╗${NC}"
  echo -e "${BLUE}║${NC} $1"
  echo -e "${BLUE}╚════════════════════════════════════════╝${NC}\n"
}

print_section() {
  echo -e "\n${CYAN}→ $1${NC}"
}

print_success() {
  echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
  echo -e "${RED}✗ $1${NC}"
  exit 1
}

print_warning() {
  echo -e "${YELLOW}⚠ $1${NC}"
}

print_info() {
  echo -e "${MAGENTA}ℹ $1${NC}"
}

show_help() {
  cat << EOF
${CYAN}Zero2Prod Development Environment${NC}

Usage: bash dev.sh [options]

Options:
  --skip-checks     Skip pre-flight checks
  --no-tests        Skip running tests before starting
  --watch          Watch and rebuild on changes
  --log-level LEVEL Set log level (debug, info, warn, error)
  --port PORT       Set server port (default: 8002)
  --help            Show this help message

Examples:
  bash dev.sh                    # Start with full checks
  bash dev.sh --skip-checks      # Skip checks and start
  bash dev.sh --watch            # Watch mode
  bash dev.sh --log-level debug  # Debug logging
  bash dev.sh --port 9000        # Custom port

EOF
}

# Pre-flight checks
run_preflight_checks() {
  if [ "$SKIP_CHECKS" = true ]; then
    print_warning "Skipping pre-flight checks"
    return
  fi

  print_section "Running Pre-flight Checks"

  # Check Rust
  if ! command -v cargo &> /dev/null; then
    print_error "Cargo not found. Please install Rust: https://rustup.rs"
  fi
  print_success "Rust/Cargo found"

  # Check PostgreSQL
  if ! command -v psql &> /dev/null; then
    print_error "PostgreSQL not found. Please install PostgreSQL"
  fi
  print_success "PostgreSQL found"

  # Check database connection
  print_info "Checking database connection..."
  if psql "$DATABASE_URL" -c "SELECT 1" &> /dev/null; then
    print_success "Database connection successful"
  else
    print_warning "Could not connect to database at $DATABASE_URL"
    print_info "Make sure PostgreSQL is running and configured correctly"
    read -p "Continue anyway? (y/N) " -n 1 -r; echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
      exit 1
    fi
  fi

  # Check code format
  print_info "Checking code format..."
  if cargo fmt -- --check &> /dev/null; then
    print_success "Code format is correct"
  else
    print_warning "Code formatting issues detected"
    echo "Run: cargo fmt"
  fi

  # Check for clippy warnings
  print_info "Running clippy..."
  if cargo clippy --all-targets --all-features -- -D warnings &> /dev/null; then
    print_success "No clippy warnings"
  else
    print_warning "Clippy warnings found"
    echo "Run: cargo clippy --all-targets --all-features"
  fi
}

# Run tests
run_tests() {
  if [ "$NO_TESTS" = true ]; then
    print_warning "Skipping tests"
    return
  fi

  print_section "Running Tests"

  if [ "$WATCH" = true ]; then
    print_info "Starting test watcher (Ctrl+C to stop)..."
    cargo watch -x test
  else
    print_info "Running test suite..."
    if cargo test --lib; then
      print_success "All tests passed"
    else
      print_error "Tests failed"
    fi
  fi
}

# Compile check
run_check() {
  print_section "Checking Code Compilation"

  if cargo check &> /dev/null; then
    print_success "Compilation check passed"
  else
    print_error "Compilation check failed"
  fi
}

# Display startup info
display_startup_info() {
  print_header "Development Environment Ready"

  echo -e "${CYAN}Configuration:${NC}"
  echo "  Port:          $PORT"
  echo "  Log Level:     $LOG_LEVEL"
  echo "  Database:      $DATABASE_URL"
  echo ""

  echo -e "${CYAN}API Endpoints:${NC}"
  echo "  Health Check:  ${GREEN}http://localhost:$PORT/health_check${NC}"
  echo "  Subscribe:     ${GREEN}POST http://localhost:$PORT/subscriptions${NC}"
  echo "  Confirm:       ${GREEN}GET http://localhost:$PORT/subscriptions/confirm${NC}"
  echo "  Newsletters:   ${GREEN}POST http://localhost:$PORT/newsletters/send-*${NC}"
  echo ""

  echo -e "${CYAN}Quick Commands (in another terminal):${NC}"
  echo "  Test:          make test"
  echo "  Lint:          cargo clippy"
  echo "  Format:        cargo fmt"
  echo "  Database:      psql -U postgres -d zero2prod"
  echo ""

  echo -e "${CYAN}Documentation:${NC}"
  echo "  Quick Start:   docs/QUICK_START.md"
  echo "  Setup Guide:   docs/SETUP_GUIDE.md"
  echo "  Architecture:  docs/08_ARCHITECTURE_AND_FLOW.md"
  echo ""
}

# Start application
start_application() {
  print_section "Starting Application"

  echo -e "${MAGENTA}════════════════════════════════════════${NC}"
  echo -e "${GREEN}Application is starting...${NC}"
  echo -e "${MAGENTA}════════════════════════════════════════${NC}\n"

  if [ "$WATCH" = true ]; then
    print_info "Watch mode enabled (Ctrl+C to stop rebuilding)"
    RUST_LOG="$LOG_LEVEL" cargo watch -x run
  else
    RUST_LOG="$LOG_LEVEL" cargo run
  fi
}

# Main execution flow
main() {
  clear

  print_header "Zero2Prod Development Environment"

  print_info "Welcome to Zero2Prod development!"
  print_info "This script will prepare your environment and start the application\n"

  # Execute steps
  run_preflight_checks
  run_check
  run_tests
  display_startup_info

  # Start application
  print_info "Starting application in 3 seconds... (Press Ctrl+C to cancel)"
  sleep 3

  start_application
}

# Trap Ctrl+C
trap 'print_info "Development environment stopped"; exit 0' INT

# Run main
main "$@"
