# Header Verifier Contract

Holds the active validator set and verifies BLS-aggregated signatures for bridge operation bundles (`hashOfHashes`). It locks operation hashes to prevent double execution and signals completion back to the calling contracts.

- **Setup:** `completeSetupPhase` fetches the genesis validator set from Chain Config and marks setup done. Only proceeds once Chain Config completed its own setup.
- **Registering operations:** `registerBridgeOps(signature, hashOfHashes, bitmap, epoch, operations)` checks setup, validates the signature against the current epoch’s validator keys, ensures the bundle hash matches, and marks each operation hash as `NotLocked`.
- **Validator rotation:** `changeValidatorSet` uses the previous epoch’s signatures to register a new validator key set (by IDs stored in Chain Config). Older epochs are pruned as `MAX_STORED_EPOCHS` is exceeded.
- **Execution locking:** `lockOperationHash(hashOfHashes, opHash, nonce)` marks an operation as `Locked` (enforcing the expected nonce). `removeExecutedHash` clears the status after completion; calling contracts typically use the wrapper in `complete_operation`.
- **Callers:** The contract expects to be the owner of MultiversX ESDT Safe and Fee Market so it can authorize bridge operations. Sovereign Forge arranges this via Chain Factory during setup.
