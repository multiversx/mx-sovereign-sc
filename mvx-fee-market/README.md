# MultiversX Fee Market

Manages bridge fees and whitelists on the MultiversX side. After setup, fee changes must be authorized through the Header Verifier.

- **Initialization:** `init(esdt_safe_address, fee)` stores the linked ESDT Safe and optional initial fee schedule.
- **Setup phase paths (owner-only):** `setFeeDuringSetupPhase`, `removeFeeDuringSetupPhase`, `addUsersToWhitelistSetupPhase`, `removeUsersFromWhitelistSetupPhase` allow configuring fees and whitelists before the bridge is opened.
- **Bridge-controlled paths:** `setFee`, `removeFee`, `distributeFees`, `addUsersToWhitelist`, `removeUsersFromWhitelist` are executed via signed operations. Each locks the operation hash in the Header Verifier before applying the change.
- **Completion:** `completeSetupPhase` marks the contract ready for bridge-controlled changes.
- **Interactions:** The ESDT Safe calls `subtractFee` (from shared endpoints) during deposits. The Header Verifier owns this contract after setup to gate operations.
