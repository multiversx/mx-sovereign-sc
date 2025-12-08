# Sovereign Bridge Smart Contracts

Contracts that deploy and run a validator-signed bridge between MultiversX and a Sovereign chain. The focus below is on how the on-chain pieces collaborate; shared helper crates sit alongside these contracts but are not described here.

## Contract Roles

- `sovereign-forge/`: entry point for sovereign creators; walks through a four-phase deployment to mint a new chain ID and request contract instances from a Chain Factory.
- `chain-factory/`: factory that clones the contract templates for each sovereign (Chain Config, Header Verifier, MultiversX ESDT Safe, Fee Market) and wires ownership to the correct controller.
- `chain-config/`: stores the sovereign’s configuration and validator set (BLS keys). Handles validator registration during the genesis phase and later updates triggered from signed bridge operations.
- `header-verifier/`: keeps the current validator set and verifies BLS-aggregated signatures for bundles of bridge operations (`hashOfHashes`). It gates execution by marking operation hashes as registered/locked/executed.
- `mvx-esdt-safe/`: MultiversX-side vault. Accepts deposits destined for the sovereign chain (burns or escrows tokens based on the chosen mechanism), emits bridge events, and executes incoming signed operations to mint/unlock tokens or perform contract calls.
- `mvx-fee-market/`: manages bridge fees and whitelists on the MultiversX side. Fee changes and whitelist updates are themselves bridge operations that must be signed and registered by the Header Verifier.
- `sov-esdt-safe/`: sovereign-side vault. Burns incoming tokens, emits the events that validators sign, and exposes admin endpoints for updating its configuration and fee sink address.
- `sov-fee-market/`: sovereign-side fee configuration; owners can set/remove fees, distribute balances, and maintain a fee whitelist.
- `interactor/`: chain-simulator flows for end-to-end tests; see `interactor/HowToRun.md`.
- `testing-sc/`: scenario test contract and fixtures.

## How the Pieces Interact

- **Bootstrapping a sovereign:** A creator calls `sovereign-forge` in four phases. The forge asks a `chain-factory` (per shard) to deploy Chain Config, MultiversX ESDT Safe, Fee Market, then Header Verifier. When everything is live, `chain-factory::completeSetupPhase` finalizes the Chain Config setup, then transfers ownership of the MultiversX-side contracts to the Header Verifier so all bridge operations are signature-gated.
- **Validator set lifecycle:** During the genesis phase, validators register BLS keys in `chain-config`. The Header Verifier pulls this set on `completeSetupPhase`. Future rotations happen through `header-verifier::changeValidatorSet`, which requires a signed operation hash from the previous epoch and the list of new validator key IDs stored in Chain Config.
- **Sovereign → MultiversX transfers:** Users deposit into `sov-esdt-safe`, which burns the tokens and emits a deposit event. Sovereign validators batch those events into a list of operations, sign the resulting `hashOfHashes`, and the Sovereign Bridge Service calls `header-verifier::registerBridgeOperations` on MultiversX. Each operation is executed through `mvx-esdt-safe::executeBridgeOps`, which locks the operation hash in the Header Verifier, mints/unlocks the needed tokens (or performs a contract call), and then signals completion so the hash is cleared.
- **MultiversX → Sovereign transfers:** Users call `mvx-esdt-safe::deposit`. The contract enforces whitelists/blacklists and fee collection, then either burns wrapped tokens or escrows native ones before emitting a deposit event. Sovereign validators observe these events and mint/unlock the corresponding assets on the sovereign chain according to their local logic.
- **Token mechanics:** `mvx-esdt-safe` supports two modes per token: burn (requires local mint/burn roles and the token to be trusted) or lock (escrow on MultiversX, unlock on return). Registering new sovereign-minted tokens on MultiversX (`registerToken`) issues a new ESDT with the sovereign prefix and maps it to the sovereign identifier; `registerNativeToken` bootstraps the sovereign chain’s own native asset.
- **Fee handling:** Deposits can require an upfront fee payment that is forwarded to `mvx-fee-market::subtractFee`. The MultiversX fee market also exposes bridge-controlled operations to set/remove fees, distribute balances, and manage a whitelist; these paths are guarded by the Header Verifier just like token transfers.
- **Pause and safeguards:** Both safes can be paused; setup phases must be completed before normal bridge operations proceed; hash locking in the Header Verifier prevents duplicate execution and enforces operation nonces.

## Siren Diagram

```
Sovereign Creator
      |
      v  deploy phases
sovereign-forge -> chain-factory ----------------------+
      |                |                               |
      |                +--> chain-config (validators)  |
      |                +--> mvx-esdt-safe (vault)      |
      |                +--> mvx-fee-market (fees)      |
      |                +--> header-verifier (owner) <--+
      |
Sovereign Chain                                          MultiversX
------------------                                       -------------------
  sov-esdt-safe (burn & emit) ----> Validators sign ----> header-verifier
                                                         |    |
  sov-fee-market (fees)     <---- fee lookups ----- mvx-esdt-safe -- mvx-fee-market
                                                         |
                                  executeBridgeOps <-----+
                                  (mint/unlock/SC calls)
```

> For more details about the Cross-Chain Execution, please take a look at the [official documentation](https://docs.multiversx.com/sovereign/cross-chain-execution). 

## Development

- Build all contracts: `sc-meta all build`
- Run contract tests from the repo root or within a contract crate: `sc-meta test`
- Simulator E2E flows: follow `interactor/HowToRun.md` (start `sc-meta cs`, delete stale `state.toml`, run the `always_deploy_setup_first` test to seed state, then execute specific tests).

## Repository Map

- `sovereign-forge/`, `chain-factory/`: deployment orchestration
- `chain-config/`, `header-verifier/`: validator management and signature verification
- `mvx-esdt-safe/`, `mvx-fee-market/`: MultiversX bridge vault and fee logic
- `sov-esdt-safe/`, `sov-fee-market/`: sovereign-side vault and fee logic
- `interactor/`, `testing-sc/`: integration and scenario tests
