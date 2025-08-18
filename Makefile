# STM32G4 Flash Programmer Makefile

.PHONY: all host firmware protocol clean help

# Default target
all: protocol host firmware
	@echo "🎉 All components built successfully!"

# Build host tool
host:
	@echo "🔨 Building host tool..."
	@cd host-tool && cargo build --release
	@echo "✅ Host tool built successfully"

# Build firmware
firmware:
	@echo "🔨 Building firmware..."
	@cd firmware && cargo build --release
	@echo "✅ Firmware built successfully"

# Build protocol library
protocol:
	@echo "🔨 Building protocol library..."
	@cd protocol && cargo build --release
	@echo "✅ Protocol library built successfully"

# Clean all build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	@rm -rf host-tool/target firmware/target protocol/target
	@echo "✅ All build artifacts cleaned"

# Development builds (debug)
dev: dev-protocol dev-host dev-firmware
	@echo "🎉 All development builds completed!"

dev-host:
	@echo "🔨 Building host tool (debug)..."
	@cd host-tool && cargo build

dev-firmware:
	@echo "🔨 Building firmware (debug)..."
	@cd firmware && cargo build

dev-protocol:
	@echo "🔨 Building protocol library (debug)..."
	@cd protocol && cargo build

# Run tests
test:
	@echo "🧪 Running tests..."
	@cd protocol && cargo test
	@cd host-tool && cargo test
	@echo "✅ Tests completed"

# Check code formatting and linting
check:
	@echo "🔍 Checking code..."
	@cd protocol && cargo clippy -- -D warnings
	@cd host-tool && cargo clippy -- -D warnings
	@cd firmware && cargo clippy -- -D warnings
	@echo "✅ Code check completed"

# Format code
fmt:
	@echo "🎨 Formatting code..."
	@cd protocol && cargo fmt
	@cd host-tool && cargo fmt
	@cd firmware && cargo fmt
	@echo "✅ Code formatted"

# Check documentation quality
docs-check:
	@echo "📝 Checking documentation..."
	@command -v markdownlint-cli2 >/dev/null 2>&1 || { echo "❌ markdownlint-cli2 not found. Install with: npm install -g markdownlint-cli2"; exit 1; }
	@markdownlint-cli2 "**/*.md" "#target" "#node_modules" || { echo "❌ Markdown linting failed"; exit 1; }
	@echo "✅ Documentation check completed"

# Check commit message format (requires git log)
commit-check:
	@echo "📋 Checking recent commit messages..."
	@command -v commitlint >/dev/null 2>&1 || { echo "❌ commitlint not found. Install with: npm install -g @commitlint/cli"; exit 1; }
	@git log --oneline -n 5 --pretty=format:"%s" | head -1 | commitlint || { echo "❌ Latest commit message doesn't follow convention"; exit 1; }
	@echo "✅ Commit message check completed"

# Run all quality checks
quality: check docs-check
	@echo "🎯 All quality checks completed!"

# Install git hooks with lefthook
hooks-install:
	@echo "🪝 Installing git hooks with lefthook..."
	@command -v lefthook >/dev/null 2>&1 || { echo "❌ lefthook not found. Install with: npm install -g lefthook"; exit 1; }
	@lefthook install
	@echo "✅ Git hooks installed successfully"

# Uninstall git hooks
hooks-uninstall:
	@echo "🗑️  Uninstalling git hooks..."
	@command -v lefthook >/dev/null 2>&1 || { echo "❌ lefthook not found"; exit 1; }
	@lefthook uninstall
	@echo "✅ Git hooks uninstalled"

# Run pre-commit hooks manually
hooks-run:
	@echo "🏃 Running pre-commit hooks..."
	@command -v lefthook >/dev/null 2>&1 || { echo "❌ lefthook not found. Install with: npm install -g lefthook"; exit 1; }
	@lefthook run pre-commit
	@echo "✅ Pre-commit hooks completed"

# Show help
help:
	@echo "STM32G4 Flash Programmer Build System"
	@echo ""
	@echo "Available targets:"
	@echo "  all           - Build all components (release)"
	@echo "  host          - Build host tool (release)"
	@echo "  firmware      - Build firmware (release)"
	@echo "  protocol      - Build protocol library (release)"
	@echo "  dev           - Build all components (debug)"
	@echo "  clean         - Clean all build artifacts"
	@echo "  test          - Run tests"
	@echo "  check         - Run clippy linting"
	@echo "  fmt           - Format code"
	@echo "  docs-check    - Check documentation quality"
	@echo "  commit-check  - Check commit message format"
	@echo "  quality       - Run all quality checks"
	@echo "  hooks-install - Install git hooks with lefthook"
	@echo "  hooks-uninstall - Uninstall git hooks"
	@echo "  hooks-run     - Run pre-commit hooks manually"
	@echo "  help          - Show this help"
