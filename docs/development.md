# Development Guide

This guide covers the development workflow and code quality tools for the STM32G4 Flash Programmer project.

## Quick Start

### 1. Setup Development Environment

Run the setup script to install all necessary tools:

```bash
./scripts/setup-dev-env.sh
```

This will install:

- **commitlint** - Commit message format checking
- **markdownlint-cli2** - Documentation quality checking
- **lefthook** - Git hooks manager
- **Rust tools** - clippy, rustfmt, embedded target

### 2. Install Git Hooks

```bash
make hooks-install
```

This installs git hooks that will automatically run quality checks before commits and pushes.

## Code Quality Tools

### Commit Message Format

We use [Conventional Commits](https://www.conventionalcommits.org/) format:

```text
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

**Examples:**

```text
feat(firmware): add W25Q128 flash support
fix(host-tool): resolve serial communication timeout
docs(readme): update installation instructions
chore(deps): update embassy to latest version
```

**Allowed types:**

- `feat` - New features
- `fix` - Bug fixes
- `docs` - Documentation changes
- `style` - Code style changes
- `refactor` - Code refactoring
- `perf` - Performance improvements
- `test` - Test additions/changes
- `chore` - Maintenance tasks
- `ci` - CI/CD changes
- `build` - Build system changes
- `revert` - Revert previous commits

**Allowed scopes:**

- `core`, `config`, `wifi`, `mqtt`, `i2c`, `charge`, `protector`, `watchdog`
- `bus`, `web-tool`, `ci`, `docs`, `tools`, `shell`, `firmware`, `deps`

### Documentation Standards

We use markdownlint to ensure consistent documentation:

- Use ATX-style headings (`#` instead of `===`)
- Use dashes for unordered lists (`-` instead of `*`)
- 2-space indentation for lists
- No trailing whitespace
- Fenced code blocks must specify language
- Images must have alt text

### Rust Code Standards

- **Formatting**: Use `cargo fmt` (enforced by rustfmt)
- **Linting**: Use `cargo clippy` with warnings as errors
- **Testing**: Write tests for new functionality

## Available Commands

### Build Commands

```bash
make all          # Build all components (release)
make dev          # Build all components (debug)
make clean        # Clean build artifacts
make test         # Run tests
```

### Quality Commands

```bash
make quality      # Run all quality checks
make check        # Run Rust clippy linting
make fmt          # Format Rust code
make docs-check   # Check documentation quality
make commit-check # Check commit message format
```

### Git Hooks Commands

```bash
make hooks-install   # Install git hooks
make hooks-uninstall # Remove git hooks
make hooks-run       # Run pre-commit hooks manually
```

## Git Hooks Workflow

### Pre-commit Hooks

Automatically run before each commit:

1. **Rust formatting check** - Ensures code is properly formatted
2. **Rust linting** - Runs clippy to catch common issues
3. **Markdown linting** - Checks documentation quality
4. **Trailing whitespace check** - Removes trailing spaces

### Commit Message Hook

Validates commit message format using commitlint.

### Pre-push Hooks

Run before pushing to remote:

1. **Tests** - Ensures all tests pass
2. **Build check** - Verifies everything compiles

## Manual Quality Checks

If you need to run checks manually:

```bash
# Check commit message format
echo "feat: add new feature" | commitlint

# Check markdown files
markdownlint-cli2 "**/*.md" "#target" "#node_modules"

# Run all pre-commit hooks
lefthook run pre-commit

# Run specific hook
lefthook run pre-commit --commands rust-fmt
```

## Troubleshooting

### Git Hooks Not Running

```bash
# Reinstall hooks
make hooks-uninstall
make hooks-install
```

### Tool Not Found Errors

```bash
# Run setup script again
./scripts/setup-dev-env.sh
```

### Commit Message Rejected

Check the commit message format and ensure it follows the conventional commits standard.

### Markdown Linting Errors

Run `make docs-check` to see specific issues and fix them according to the markdownlint rules.

## CI/CD Integration

The same quality checks run automatically in GitHub Actions:

- **Commit Check** - Validates commit messages in PRs
- **Documentation Check** - Validates markdown files
- **Rust Checks** - Formatting, linting, and testing
- **Build Verification** - Ensures everything compiles

## Configuration Files

- `lefthook.yml` - Git hooks configuration
- `commitlint.config.cjs` - Commit message rules
- `.markdownlint-cli2.yaml` - Documentation linting rules
- `.github/workflows/ci.yml` - CI/CD pipeline
- `docs/development.md` - This development guide
