# Repository Guidelines

## Project Structure & Module Organization
Each MultiversX smart contract lives in its own crate (e.g. `mvx-fee-market`, `sov-esdt-safe`, `header-verifier`) with logic in `src/` and generated artifacts in `wasm/` and `output/`. Shared crates in `common/` (`common-utils`, `structs`, `fee-common`, etc.) feed reusable modules, meta crates (`*/meta`) emit ABI and deployable bundles, `interactor/` hosts integration flows, and `chain-config/` keeps scenario tests and deployment presets aligned.

## Build, Test, and Development Commands
Build and compile the smart contracts with `sc-meta all build`, which refreshes `multiversx.json` and the matching `output/` bundle. Execute blackbox tests with `sc-meta test` either inside one contract crate or at the root of the repo. Simulator interaction flows follow `interactor/HowToRun.md`: start `sc-meta cs start`, delete stale `state.toml`, run the bootstrap deployment (`cargo test --package rust-interact --test always_deploy_setup_first --all-features -- deploy_setup --exact --show-output`), then execute focused scenarios.

## Coding Style & Naming Conventions
Follow rustfmt defaults (4-space indentation, trailing commas) and run `cargo fmt --all` before submitting changes. Contract modules and files use `snake_case`, traits stay in `UpperCamelCase`, and constants follow the MultiversX screaming-snake style (`ESDT_SAFE_ADDRESS_NOT_SET`). Lint with `cargo clippy --workspace --all-targets -- -D warnings` to keep endpoint traits and storage modules aligned.

## Testing Guidelines
Prefer `multiversx-sc-scenario` tests for endpoint coverage, naming them after the contract and behavior (e.g. `fee_market_complete_setup.rs`). Seed fixtures through `common-test-setup` helpers to keep any contract interaction easy to use across all the smart contracts. For simulator-backed tests, follow the bootstrap steps in `interactor/HowToRun.md` so the common state is seeded before running per-file suites. Include negative-path assertions for guard checks and document any skipped cases inline. When touching deployment presets, rerun `chain-config/tests` to confirm serialized output still mirrors `sc-config.toml`.

## Commit & Pull Request Guidelines
CRITICAL: DO NOT COMMIT ANYTHING YOURSELF
