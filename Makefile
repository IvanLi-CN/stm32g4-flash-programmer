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

# Show help
help:
	@echo "STM32G4 Flash Programmer Build System"
	@echo ""
	@echo "Available targets:"
	@echo "  all       - Build all components (release)"
	@echo "  host      - Build host tool (release)"
	@echo "  firmware  - Build firmware (release)"
	@echo "  protocol  - Build protocol library (release)"
	@echo "  dev       - Build all components (debug)"
	@echo "  clean     - Clean all build artifacts"
	@echo "  test      - Run tests"
	@echo "  check     - Run clippy linting"
	@echo "  fmt       - Format code"
	@echo "  help      - Show this help"
