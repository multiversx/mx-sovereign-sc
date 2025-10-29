# MultiversX Sovereign Bridge Smart Contracts - Copilot Instructions

## Repository Overview

This repository contains **Sovereign Bridge Smart Contracts** for the MultiversX blockchain ecosystem. It is a **Rust-based smart contract workspace** using the MultiversX smart contract framework (v0.57.1) with 7 primary smart contracts and shared common libraries.

**Key Statistics:**
- Language: Rust
- Framework: MultiversX SC v0.57.1
- Smart Contracts: 7 (header-verifier, fee-market, sov-esdt-safe, mvx-esdt-safe, chain-config, chain-factory, testing-sc)
- Common Libraries: 8 (in `common/` directory)
- Build Target: wasm32-unknown-unknown
- Rust Toolchain: stable (currently 1.90.0)

## Prerequisites and Environment Setup

**CRITICAL: Always install these prerequisites before building:**

1. **Rust toolchain with wasm32 target:**
   ```bash
   rustup target add wasm32-unknown-unknown
   ```
   This MUST be installed before any build operations. Build will fail without it.

2. **MultiversX sc-meta tool (v0.62.1 or compatible):**
   ```bash
   cargo install multiversx-sc-meta --locked
   ```
   This tool is required for building contracts. Install takes ~3-4 minutes.

3. **Optional but recommended: wasm-opt**
   - Not required for builds but recommended for production
   - Builds will show "Warning: wasm-opt not installed" but still succeed

## Build Instructions

**Build Commands (in order of priority):**

### 1. Build All Contracts
**Command:** `sc-meta all build --locked`
- **Location:** Run from repository root
- **Time:** ~50 seconds from clean build, ~10-15 seconds incremental
- **Output:** Generates WASM files in `*/output/` directories (one per contract)
- **ALWAYS use `--locked` flag** to ensure reproducible builds with locked dependencies

### 2. Clean Build Artifacts
**Command:** `sc-meta all clean`
- Removes output files from all contract directories
- Does NOT clean the cargo target directory
- For full clean: `cargo clean` (removes 3.4GB, takes ~5 seconds)

### 3. Generate ABI Files Only
**Command:** `sc-meta all abi`
- Generates contract ABI JSON files without building WASM
- Faster than full build (~5 seconds)
- Used for interface generation and documentation

### 4. Generate/Verify Proxy Files
**Command:** `sc-meta all proxy --compare`
- Generates proxy Rust code from contract ABIs
- `--compare` flag verifies generated proxies match existing ones
- Used in CI (proxy-compare workflow)
- Takes ~2-3 seconds

## Testing Instructions

### Run All Tests
**Command:** `cargo test --locked`
- **Time:** ~50 seconds (including compilation)
- **Test Results:** 62 tests pass (37 in mvx-esdt-safe, 11 in header-verifier, 7 in fee-market, 7 in sov-esdt-safe)
- 15 tests in interactor are ignored (require live network/simulator)
- **ALWAYS use `--locked`** to use locked dependencies

### Test Structure:
- **Unit Tests:** In `*/tests/` directories using MultiversX scenario testing framework
- **Blackbox Tests:** Named `*_blackbox_*.rs` - integration tests with full contract deployment
- **Interactor Tests:** In `interactor/tests/` - require network connection (normally ignored)

## Linting and Formatting

### Code Formatting
**Command:** `cargo fmt`
- **Check only:** `cargo fmt --check` (exits with error if formatting needed)
- Repository has minor formatting issues in interactor files (trailing whitespace/newlines)
- CI does not enforce formatting currently

### Linting with Clippy
**Command:** `cargo clippy --locked`
- **Time:** ~25 seconds after build
- **Current Status:** 1 warning in `common/common-test-setup` (clippy::manual_contains)
- **Fix suggestions:** `cargo clippy --fix --lib -p common-test-setup`

## Continuous Integration

The repository has 3 main CI workflows (in `.github/workflows/`):

### 1. actions.yml (Main CI)
- **Trigger:** Push to main/feat/* branches, PRs, manual dispatch
- **Uses:** `multiversx/mx-sc-actions/.github/workflows/contracts.yml@v4.2.2`
- **Parameters:**
  - rust-toolchain: stable
  - enable-interactor-tests: true
  - coverage-args: `--ignore-filename-regex='/.cargo/git' --output ./coverage.md`

### 2. on_pull_request_build_contracts.yml
- **Trigger:** All pull requests
- **Uses:** `multiversx/mx-sc-actions/.github/workflows/reproducible-build.yml@v4.2.2`
- **Image:** v10.0.0
- **Purpose:** Ensures reproducible contract builds

### 3. proxy-compare.yml
- **Trigger:** Push to master branch, PRs
- **Steps:**
  1. Install Rust with wasm32-unknown-unknown target
  2. Install multiversx-sc-meta
  3. Run `sc-meta all proxy --compare`
- **Purpose:** Verifies proxy files are up-to-date
- **Note:** Uses 'master' branch while main CI uses 'main/feat/*' branches

### 4. release.yml
- **Trigger:** Release published
- **Attaches build artifacts to release**

## Project Structure

### Smart Contract Directories
Each contract follows standard MultiversX structure:

```
<contract-name>/
├── Cargo.toml              # Contract dependencies (multiversx-sc = "0.57.1")
├── multiversx.json         # Contains: {"language": "rust"}
├── sc-config.toml          # Multi-contract configuration (if applicable)
├── src/                    # Contract source code
│   └── lib.rs             # Main contract module
├── meta/                   # Meta crate for building
│   └── Cargo.toml
├── wasm*/                  # WASM build crates (auto-generated)
├── output/                 # Built artifacts (.wasm, .abi.json, .mxsc.json)
└── tests/                  # Blackbox/scenario tests (if applicable)
```

**Contracts with multiple WASM outputs (using sc-config.toml):**
- fee-market: 3 variants (main, full, view)
- header-verifier: 3 variants (main, full, view)
- sov-esdt-safe: 3 variants (main, full, view)
- mvx-esdt-safe: 3 variants (main, full, view)
- testing-sc: 3 variants (main, full, view)

### Common Libraries (`common/` directory)
Shared code used across contracts:
- **common-test-setup**: Test utilities and setup helpers
- **cross-chain**: Cross-chain communication structures
- **error-messages**: Centralized error message constants
- **proxies**: Auto-generated contract proxies
- **setup-phase**: Setup phase logic
- **structs**: Shared data structures
- **token-whitelist**: Token whitelist module
- **utils**: Utility functions

### Interactor (`interactor/` directory)
- CLI tool for contract interaction
- Config: `config.toml` (can target simulator or real network)
- Tests require network connection - normally ignored in CI

### Build Output Structure
After `sc-meta all build`:
- Each contract creates `output/` directory
- Contains: `.wasm`, `.abi.json`, `.mxsc.json`, `.imports.json` files
- Example sizes: 6-40KB for WASM files, view contracts ~700 bytes

## Common Issues and Workarounds

### Issue: Build fails with "can't find crate for `core`"
**Cause:** wasm32-unknown-unknown target not installed
**Solution:** `rustup target add wasm32-unknown-unknown` (ALWAYS do this first)

### Issue: "sc-meta: command not found"
**Cause:** multiversx-sc-meta not installed
**Solution:** `cargo install multiversx-sc-meta --locked` (takes 3-4 minutes)

### Issue: "Warning: wasm-opt not installed"
**Cause:** wasm-opt binary not in PATH
**Impact:** Non-critical - builds succeed without optimization
**Solution:** Install binaryen toolkit (optional)

### Issue: Incremental build issues
**Solution:** Run `cargo clean && sc-meta all clean` then rebuild
**Time cost:** Full rebuild ~50 seconds

## Version Information

- **MultiversX SC Framework:** 0.57.1 (contracts), 0.62.1 (meta tool)
- **Rust:** stable channel (tested with 1.90.0)
- **Cargo:** 1.90.0
- **Build Image (CI):** v10.0.0 (for reproducible builds)

## Quick Reference Commands

```bash
# First-time setup
rustup target add wasm32-unknown-unknown
cargo install multiversx-sc-meta --locked

# Build contracts
sc-meta all build --locked                    # ~50s clean, ~10s incremental

# Run tests
cargo test --locked                           # ~50s

# Lint/Format
cargo clippy --locked                         # ~25s
cargo fmt                                     # ~2s

# Clean
sc-meta all clean                             # Clean output dirs
cargo clean                                   # Full clean (3.4GB)

# CI validation
sc-meta all proxy --compare                   # Verify proxies
```

## Important Notes

1. **ALWAYS use `--locked` flag** with cargo and sc-meta commands for reproducible builds
2. **wasm32-unknown-unknown target is mandatory** - install before first build
3. **Build times:** Clean ~50s, incremental ~10-15s, tests ~50s
4. **Ignored tests:** 15 interactor tests require network connection
5. **CI uses reusable workflows** from multiversx/mx-sc-actions repository
6. **Multiple WASM outputs:** Some contracts generate 3 variants (main, full, view)
7. **Trust these instructions:** Only search for additional information if these instructions are incomplete or incorrect

## File Locations Reference

- **Root config:** `Cargo.toml`, `.gitignore`
- **CI workflows:** `.github/workflows/*.yml`
- **Contract configs:** `*/Cargo.toml`, `*/sc-config.toml`, `*/multiversx.json`
- **Build outputs:** `*/output/*.wasm`, `*/output/*.abi.json`
- **Common code:** `common/*/src/`
- **Interactor:** `interactor/src/`, `interactor/config.toml`
