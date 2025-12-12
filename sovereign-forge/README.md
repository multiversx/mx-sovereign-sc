# Sovereign Forge

Orchestrates the four-phase deployment of a sovereign’s MultiversX-side contracts via Chain Factory and tracks chain IDs per creator.

- **Initialization:** `init(opt_deploy_cost)` sets the required EGLD deposit for deployments and pauses the contract.
- **Registering factory/trusted tokens:** `registerChainFactory(shard_id, address)` wires a Chain Factory per shard. `registerTrustedToken` stores token IDs allowed for burn/lock mechanics downstream.
- **Deployment phases (caller = sovereign creator):**
  - `deployPhaseOne(opt_preferred_chain_id, config)` → deploy Chain Config and reserve a chain ID.
  - `deployPhaseTwo(opt_config)` → deploy MultiversX ESDT Safe with the reserved prefix.
  - `deployPhaseThree(fee)` → deploy MultiversX Fee Market linked to the ESDT Safe.
  - `deployPhaseFour()` → deploy Header Verifier with references to the other contracts.
- **Finishing setup:** `completeSetupPhase` triggers Chain Factory to run the per-contract `completeSetupPhase` calls and transfer ownership to the Header Verifier. Marks the sovereign as setup-complete.
- **Interactions:** Drives Chain Factory to clone templates. Validators and the Header Verifier rely on the chain ID generated here to namespace token IDs and contract lookups.
