# MultiversX Sovereign Bridge Smart Contracts - Copilot Instructions

## Repository Overview

This repository contains the **Sovereign Bridge Smart Contracts** for MultiversX blockchain. It implements a cross-chain bridge system between MultiversX mainnet and sovereign chains.

**Project Type:** Rust Smart Contracts for MultiversX blockchain  
**Framework:** MultiversX Smart Contract SDK
**Language:** Rust (toolchain 1.87)  
**Target:** WebAssembly (wasm32-unknown-unknown)  
**Repository Size:** ~130 Rust source files  
**Build Tool:** `sc-meta` (multiversx-sc-meta)

## Project Structure

### Core Smart Contracts (9 contracts in root)
- **chain-config/** - Chain configuration and validator management
- **chain-factory/** - Factory for creating chain instances
- **header-verifier/** - Cross-chain header verification
- **mvx-esdt-safe/** - MultiversX side ESDT token safe
- **mvx-fee-market/** - MultiversX fee market management
- **sov-esdt-safe/** - Sovereign chain ESDT token safe
- **sov-fee-market/** - Sovereign chain fee market
- **sovereign-forge/** - Main sovereign chain orchestration contract
- **testing-sc/** - Testing utilities contract

### Common Libraries (in common/)
- **common-interactor/** - Shared interactor utilities
- **common-test-setup/** - Shared test setup and helpers
- **common-utils/** - Common utility functions
- **cross-chain/** - Cross-chain communication utilities
- **custom-events/** - Event definitions
- **error-messages/** - Centralized error messages
- **fee-common/** - Fee-related shared logic
- **proxies/** - Contract proxies for interaction
- **setup-phase/** - Setup phase management module
- **structs/** - Shared data structures
- **tx-nonce/** - Transaction nonce management

### Interactor Tests (interactor/)
Contains integration tests that run against Chain Simulator. See dedicated section below.

### Configuration Files (per contract)
Each contract directory contains:
- `Cargo.toml` - Main package configuration
- `multiversx.json` - Language identifier (Rust)
- `sc-config.toml` - Proxy generation configuration
- `src/` - Contract source code
- `tests/` - Blackbox tests
- `wasm/` - WASM build configuration
- `meta/` - Build metadata and code generation
- `output/` - Compiled WASM outputs (auto-generated, gitignored)

## Build Instructions

### Prerequisites
**ALWAYS install sc-meta before building:**
```bash
cargo install multiversx-sc-meta --locked
```
Installation is REQUIRED for all build operations.

### Build All Contracts
```bash
sc-meta all build
```
**Output:** WASM files in each `{contract}/output/` directory  
**Warning:** You will see "wasm-opt not installed" warnings - these are safe to ignore. The build succeeds without wasm-opt.

**Build artifacts created:**
- `{contract}/output/{contract-name}.wasm` - Compiled contract
- `{contract}/output/{contract-name}.imports.json` - Import definitions
- `{contract}/output/{contract-name}.mxsc.json` - Contract metadata

### Build Single Contract
```bash
cd {contract-directory}/meta
cargo run build
```

### Clean Build Artifacts
```bash
sc-meta all clean
```
Removes all `output/` directories. Use before fresh builds or to free disk space.

## Testing

### Blackbox Tests (Fast Unit Tests)
**Run all blackbox tests:**
```bash
cargo test --workspace
```
**Note:** These are standard Rust unit tests using the MultiversX scenario testing framework.

**Run specific contract tests:**
```bash
cargo test --package chain-config --test chain_config_blackbox_tests
```

### Chain Simulator Integration Tests
The interactor tests require a running Chain Simulator and follow a specific workflow.

**CRITICAL: ALWAYS read `interactor/HowToRun.md` before running or modifying interactor tests.**

The HowToRun.md file contains the complete, authoritative workflow including:
- Chain Simulator startup and state management
- Required test execution sequence (deployment test must run first)
- Exact command syntax for running tests
- Troubleshooting guide for common issues

**Do not rely on this summary alone** - always reference `interactor/HowToRun.md` for the most up-to-date instructions.

## Linting and Code Quality

### Clippy
```bash
cargo clippy --all-targets --all-features
```
**Output:** Clippy will complete with no errors (clean codebase)

### Proxy Comparison (Validates generated proxies match committed ones)
```bash
sc-meta all proxy --compare
```  
This command is used in CI to ensure proxy files are up to date.

## GitHub Actions CI Workflows

### Primary CI (actions.yml)
**Triggers:** Push to main, feat/* branches, PRs, manual dispatch  
**Rust Toolchain:** 1.87  
**Actions:** Uses `multiversx/mx-sc-actions/.github/workflows/contracts.yml@v4.2.2`
- Builds all contracts
- Runs tests (including interactor tests with `enable-interactor-tests: true`)
- Generates coverage report
- Validates code quality

### Pull Request Build (on_pull_request_build_contracts.yml)
**Triggers:** All pull requests  
**Actions:** Reproducible build using Docker image `v10.0.0`

### Proxy Compare (proxy-compare.yml)
**Triggers:** Push to master, PRs  
**Actions:** Validates generated proxies match committed files

### Release Build (release.yml)
**Triggers:** Release published  
**Actions:** Reproducible build + attach artifacts to release

## Common Issues and Workarounds

### "wasm-opt not installed" Warning
**Status:** Safe to ignore. Contracts build successfully without wasm-opt optimization.

### Build Failures After Dependency Changes
**Solution:** Run `cargo clean` then rebuild:
```bash
cargo clean
sc-meta all build
```

### Interactor Tests Failing
**Root Causes:**
1. Chain simulator not running → Start with `sc-meta cs start`
2. Missing deployment → Run deploy_setup test first
3. Stale state → Delete state.toml and re-deploy

### Proxy Out of Sync
If proxy compare fails in CI:
```bash
sc-meta all proxy
git add common/proxies/
git commit -m "Update proxies"
```

## Development Workflow

### Making Code Changes
1. **ALWAYS build after changes:**
```bash
sc-meta all build
```

2. **ALWAYS run tests:**
```bash
cargo test --workspace
```

3. **For interactor test changes, run full chain simulator workflow**

4. **Before committing, validate:**
```bash
cargo clippy --all-targets --all-features
sc-meta all proxy --compare
```

### Adding New Contract
1. Add to workspace in root `Cargo.toml`
2. Contract must have: `src/`, `meta/`, `wasm/`, `tests/` directories
3. Include `multiversx.json` and `sc-config.toml`
4. Generate proxy if needed: `sc-meta all proxy`

### Modifying Contract Endpoints
**ALWAYS regenerate proxies after endpoint changes:**
```bash
sc-meta all proxy
```
This updates files in `common/proxies/src/`.

## Key Dependencies

All contracts share common modules from the `common/` directory. When modifying common modules, test all contracts that depend on them.

## Repository Root Files
- `.github/` - GitHub Actions workflows
- `.gitignore` - Excludes target/, output/, and build artifacts
- `Cargo.toml` - Workspace definition with 19 members (9 contracts + 9 meta + interactor)
- `Cargo.lock` - Locked dependencies (committed)
- `README.md` - Basic repository description

## Trust These Instructions

These instructions are comprehensive and validated. When working in this repository:
- Follow build and test sequences exactly as documented
- Only search for additional information if these instructions are incomplete or incorrect
- The documented commands and workflows have been tested and verified
- CI failures usually indicate deviation from documented workflow, not bugs in CI
