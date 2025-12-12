# Chain Factory Contract

Factory that clones the sovereign contract templates on MultiversX and wires ownership to the correct controllers.

- **Initialization:** `init` receives the Sovereign Forge address plus template addresses for Chain Config, Header Verifier, MultiversX ESDT Safe, and Fee Market. Templates are required to be valid smart contract addresses.
- **Deploying per sovereign:** Only admins (typically the Sovereign Forge) may call:
  - `deploySovereignChainConfigContract(opt_config)`
  - `deployEsdtSafe(sovereign_owner, sov_prefix, opt_config)`
  - `deployFeeMarket(esdt_safe_address, fee)`
  - `deployHeaderVerifier(sovereign_contracts)`
  Each returns the fresh contract address.
- **Completing setup:** `completeSetupPhase` calls `completeSetupPhase` on Chain Config, then transfers ownership of Chain Config, MultiversX ESDT Safe, and Fee Market to the Header Verifier, and finally completes their setup phases. This makes the Header Verifier the gatekeeper for bridge operations.
- **Interactions:** Sovereign Forge drives deployments through this factory. After phase four, the factory’s setup completion hooks finish wiring the sovereign’s contract suite.
