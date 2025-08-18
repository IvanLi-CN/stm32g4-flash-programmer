#!/bin/bash

# STM32G4 Flash Programmer Development Environment Setup Script
# This script installs all necessary tools for code quality checks

set -e

echo "🚀 Setting up STM32G4 Flash Programmer development environment..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Install Node.js tools
install_node_tools() {
    print_status "Installing Node.js tools..."
    
    if ! command_exists npm; then
        print_error "npm not found. Please install Node.js first."
        print_status "Visit: https://nodejs.org/"
        exit 1
    fi
    
    # Install commitlint
    if ! command_exists commitlint; then
        print_status "Installing commitlint..."
        npm install -g @commitlint/cli
        print_success "commitlint installed"
    else
        print_success "commitlint already installed"
    fi
    
    # Install markdownlint
    if ! command_exists markdownlint-cli2; then
        print_status "Installing markdownlint-cli2..."
        npm install -g markdownlint-cli2
        print_success "markdownlint-cli2 installed"
    else
        print_success "markdownlint-cli2 already installed"
    fi
    
    # Install lefthook
    if ! command_exists lefthook; then
        print_status "Installing lefthook..."
        npm install -g lefthook
        print_success "lefthook installed"
    else
        print_success "lefthook already installed"
    fi
}

# Install Rust tools
install_rust_tools() {
    print_status "Checking Rust tools..."
    
    if ! command_exists cargo; then
        print_error "Cargo not found. Please install Rust first."
        print_status "Visit: https://rustup.rs/"
        exit 1
    fi
    
    # Check if clippy is available
    if ! rustup component list --installed | grep -q clippy; then
        print_status "Installing clippy..."
        rustup component add clippy
        print_success "clippy installed"
    else
        print_success "clippy already installed"
    fi
    
    # Check if rustfmt is available
    if ! rustup component list --installed | grep -q rustfmt; then
        print_status "Installing rustfmt..."
        rustup component add rustfmt
        print_success "rustfmt installed"
    else
        print_success "rustfmt already installed"
    fi
    
    # Add thumbv7em-none-eabihf target for embedded development
    if ! rustup target list --installed | grep -q thumbv7em-none-eabihf; then
        print_status "Installing thumbv7em-none-eabihf target..."
        rustup target add thumbv7em-none-eabihf
        print_success "thumbv7em-none-eabihf target installed"
    else
        print_success "thumbv7em-none-eabihf target already installed"
    fi
}

# Setup git hooks
setup_git_hooks() {
    print_status "Setting up git hooks with lefthook..."
    
    if ! command_exists lefthook; then
        print_error "lefthook not found. Please install it first."
        exit 1
    fi
    
    # Install lefthook hooks
    lefthook install
    print_success "Git hooks installed with lefthook"
}

# Verify installation
verify_installation() {
    print_status "Verifying installation..."
    
    local all_good=true
    
    # Check Node.js tools
    if command_exists commitlint; then
        print_success "✓ commitlint: $(commitlint --version)"
    else
        print_error "✗ commitlint not found"
        all_good=false
    fi
    
    if command_exists markdownlint-cli2; then
        print_success "✓ markdownlint-cli2: $(markdownlint-cli2 --version)"
    else
        print_error "✗ markdownlint-cli2 not found"
        all_good=false
    fi
    
    if command_exists lefthook; then
        print_success "✓ lefthook: $(lefthook version)"
    else
        print_error "✗ lefthook not found"
        all_good=false
    fi
    
    # Check Rust tools
    if command_exists cargo; then
        print_success "✓ cargo: $(cargo --version)"
    else
        print_error "✗ cargo not found"
        all_good=false
    fi
    
    if $all_good; then
        print_success "🎉 All tools installed successfully!"
        echo ""
        print_status "Next steps:"
        echo "  1. Run 'make hooks-run' to test pre-commit hooks"
        echo "  2. Run 'make quality' to run all quality checks"
        echo "  3. Start developing with confidence! 🚀"
    else
        print_error "Some tools are missing. Please check the installation."
        exit 1
    fi
}

# Main execution
main() {
    echo ""
    print_status "Starting development environment setup..."
    echo ""
    
    install_node_tools
    echo ""
    
    install_rust_tools
    echo ""
    
    setup_git_hooks
    echo ""
    
    verify_installation
}

# Run main function
main "$@"
