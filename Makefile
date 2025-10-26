.PHONY: build run test clean install check lint format dev-setup

# Default target
all: build

# Build the project
build:
	cargo build --release

# Run the server
run-server:
	cargo run --bin server

# Run the GUI
run-gui:
	cargo run --bin gui

# Run tests
test:
	cargo test

# Run tests with output
test-verbose:
	cargo test -- --nocapture

# Clean build artifacts
clean:
	cargo clean

# Install dependencies (development setup)
dev-setup:
	rustup component add clippy rustfmt
	cargo install cargo-watch

# Check code without building
check:
	cargo check

# Lint the code
lint:
	cargo clippy -- -D warnings

# Format the code
format:
	cargo fmt

# Watch for changes and rebuild
watch:
	cargo watch -x check

# Watch and run tests
test-watch:
	cargo watch -x test

# Create a new migration
migration:
	@read -p "Enter migration name: " name; \
	mkdir -p migrations; \
	timestamp=$$(date +%Y%m%d%H%M%S); \
	touch migrations/$${timestamp}_$${name}.sql

# Setup database
db-setup:
	mkdir -p data
	sqlite3 data/dj_system.db < migrations/20240101000001_initial.sql

# Reset database
db-reset:
	rm -f data/dj_system.db
	$(MAKE) db-setup

# Run development server with auto-reload
dev:
	cargo watch -x 'run --bin server'

# Run GUI in development mode
dev-gui:
	cargo watch -x 'run --bin gui'

# Install the application system-wide
install: build
	sudo cp target/release/server /usr/local/bin/dj-system-server
	sudo cp target/release/gui /usr/local/bin/dj-system-gui

# Create a release build with optimizations
release:
	cargo build --release
	strip target/release/server target/release/gui

# Run security audit
audit:
	cargo audit

# Generate documentation
docs:
	cargo doc --no-deps --open

# Package for distribution
package: release
	mkdir -p dist
	cp target/release/server dist/
	cp target/release/gui dist/
	cp README.md dist/
	cp .env.example dist/
	tar -czf dist/dj-system.tar.gz -C dist .

# Docker build (if using Docker)
docker-build:
	docker build -t dj-system .

# Show help
help:
	@echo "Available commands:"
	@echo "  build       - Build the project"
	@echo "  run-server  - Run the API server"
	@echo "  run-gui     - Run the GUI application"
	@echo "  test        - Run tests"
	@echo "  clean       - Clean build artifacts"
	@echo "  dev-setup   - Install development dependencies"
	@echo "  check       - Check code without building"
	@echo "  lint        - Run clippy linter"
	@echo "  format      - Format code with rustfmt"
	@echo "  watch       - Watch for changes and rebuild"
	@echo "  test-watch  - Watch for changes and run tests"
	@echo "  migration   - Create a new database migration"
	@echo "  db-setup    - Initialize database"
	@echo "  db-reset    - Reset database"
	@echo "  dev         - Run development server with auto-reload"
	@echo "  dev-gui     - Run GUI in development mode"
	@echo "  install     - Install system-wide"
	@echo "  release     - Create optimized release build"
	@echo "  audit       - Run security audit"
	@echo "  docs        - Generate documentation"
	@echo "  package     - Package for distribution"
	@echo "  help        - Show this help message"