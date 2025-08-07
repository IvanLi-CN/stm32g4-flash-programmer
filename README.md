# STM32G4 Flash Programmer

A comprehensive tool suite for programming W25Q128 Flash memory via ST-Link on STM32G431CB microcontroller.

## 🚀 Features

- **Flash Programmer**: Host tool for programming W25Q128 flash memory
- **Flash Test Image**: Embedded firmware for STM32G431CB with Embassy async framework
- **PNG to Bitmap Converter**: Utility for converting PNG images to bitmap format
- **Automated CI/CD**: GitHub Actions for testing, building, and releasing
- **Code Quality**: Automated formatting, linting, and commit message validation

## 📁 Project Structure

```
├── flash_programmer/     # Host tool for flash programming
├── flash_test_image/     # STM32G4 embedded firmware
├── tools/               # Utility scripts and tools
├── test_data/           # Test data files
├── .github/             # GitHub Actions workflows
├── commitlint.config.cjs # Commit message linting configuration
└── lefthook.yml         # Git hooks configuration
```

## 🛠️ Development Setup

### Prerequisites

- Rust toolchain (stable)
- Node.js (for commit linting)
- probe-rs (for flashing STM32)
- lefthook (for git hooks)

### Installation

1. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd stm32g4-flash-programmer
   ```

2. **Install Rust target for STM32G4**:
   ```bash
   rustup target add thumbv7em-none-eabihf
   ```

3. **Install Node.js dependencies**:
   ```bash
   npm install
   ```

4. **Install and activate git hooks**:
   ```bash
   lefthook install
   ```

### Building

- **Flash Programmer** (host tool):
  ```bash
  cd flash_programmer
  cargo build --release
  ```

- **Flash Test Image** (STM32G4 firmware):
  ```bash
  cd flash_test_image
  cargo build --target thumbv7em-none-eabihf --release
  ```

- **PNG Converter**:
  ```bash
  cargo build --release
  ```

### Flashing STM32G4

```bash
cd flash_test_image
probe-rs run --chip STM32G431CBUx target/thumbv7em-none-eabihf/release/flash_test_image
```

## 🔧 Git Hooks

This project uses [lefthook](https://github.com/evilmartians/lefthook) for git hooks:

- **pre-commit**: Runs `cargo fmt` and `cargo clippy` on Rust files
- **commit-msg**: Validates commit messages using commitlint

### Commit Message Format

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<scope>): <description>

<body>
```

**Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`, `ci`, `build`, `revert`

**Examples**:
- `feat: add W25Q128 flash programming support`
- `fix(flash): resolve SPI communication timeout`
- `docs: update README with setup instructions`

## 🚀 CI/CD

### GitHub Actions Workflows

1. **CI** (`.github/workflows/ci.yml`):
   - Runs on push to main and feature branches
   - Performs code checking, testing, linting, and building
   - Validates commit messages on pull requests

2. **Dependencies** (`.github/workflows/dependencies.yml`):
   - Weekly security audits
   - Dependency update checks
   - Automated dependency updates (manual trigger)

3. **Release** (`.github/workflows/release.yml`):
   - Triggered on version tags (`v*`)
   - Builds release artifacts for all projects
   - Creates GitHub releases with binaries

### Dependabot

Automated dependency updates are configured for:
- Rust dependencies (main project, flash_programmer, flash_test_image)
- Node.js dependencies (development tools)
- GitHub Actions

## 📦 Release Process

1. **Create a version tag**:
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

2. **GitHub Actions will automatically**:
   - Build all release artifacts
   - Generate checksums
   - Create a GitHub release
   - Upload binaries and documentation

## 🧪 Testing

Run tests for the main project:
```bash
cargo test
```

The embedded projects (flash_programmer and flash_test_image) are primarily integration-tested through the CI pipeline.

## 📄 License

[Add your license information here]

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feat/amazing-feature`)
3. Make your changes
4. Ensure all tests pass and code is formatted
5. Commit using conventional commit format
6. Push to your branch (`git push origin feat/amazing-feature`)
7. Open a Pull Request

The git hooks will automatically:
- Format your Rust code
- Run clippy lints
- Validate your commit messages
