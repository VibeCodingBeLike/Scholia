# Contributing to Scholia

Thank you for your interest in contributing to **Scholia**! We welcome contributions from developers, technical writers, and academic researchers who want to make MLA writing seamless, beautiful, and strictly compliant.

---

## 🏛️ Guiding Philosophy: Strict MLA Enforcement
The foundational principle of this project is:
> **The user must never be able to break MLA formatting.**

When contributing new features or UI controls:
- Do **not** add arbitrary font pickers, arbitrary font size selectors, or custom margin adjusters.
- Any new typographical options must be strictly within approved MLA 9th Edition guidelines.
- Features that automate or validate MLA rules (e.g. citation builders, works cited sorters, heading structure) are warmly encouraged.

---

## 🛠️ Development Setup

### Prerequisites
1. **Rust 1.80+**: Install via [rustup.rs](https://rustup.rs/):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. **Platform-Specific Dependencies** (Linux only):
   ```bash
   sudo apt update
   sudo apt install -y libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libasound2-dev libgtk-3-dev
   ```

### Building & Running Locally
```bash
# Clone the repository
git clone https://github.com/Scholia/Scholia.git
cd Scholia

# Run the app locally in development mode
cargo run

# Run automated tests
cargo test

# Check code formatting
cargo fmt --all -- --check

# Run Clippy lints
cargo clippy --all-targets -- -D warnings
```

---

## 🧪 Testing Your Changes
Always verify that all existing tests pass before submitting a pull request:
```bash
cargo test
```
If you add new features (e.g. new MLA rules, citation elements, or export formats), please include corresponding unit or integration tests in `tests/mla_tests.rs`.

---

## 📦 Packaging Scripts
- Windows portable: `powershell -File .\build_portable.ps1`
- Windows installer: `powershell -File .\build_release.ps1`
- macOS bundle: `./installers/macos/build_dmg.sh`
- Linux Debian: `./installers/linux/build_deb.sh`

---

## 📜 Pull Request Process
1. Fork the repo and create your branch from `main`:
   ```bash
   git checkout -b feature/my-new-feature
   ```
2. Make your commits with clear, conventional messages (`feat: ...`, `fix: ...`, `docs: ...`).
3. Ensure formatting and clippy pass cleanly:
   ```bash
   cargo fmt --all
   cargo clippy --all-targets
   ```
4. Push to your fork and submit a Pull Request using our PR template.
