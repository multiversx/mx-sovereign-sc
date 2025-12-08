# MultiversX ESDT Safe

MultiversX-side vault for the bridge. It accepts deposits heading to the sovereign chain, executes signed operations coming from the sovereign side, and manages token registration plus burn/lock mechanics.

- **Initialization:** `init(sovereign_owner, sovereign_forge_address, sov_token_prefix, opt_config)` sets the sovereign owner/admin, validates the sovereign token prefix, stores config (whitelist/blacklist, gas limits, max amounts), and starts paused.
- **Deposits (MultiversX → Sovereign):** `deposit(to, optTransferData)` enforces pause/blacklists/whitelists, charges bridge fees via the Fee Market, then burns or locks tokens based on the configured mechanism before emitting a deposit event.
- **Execution (Sovereign → MultiversX):** `executeBridgeOps(hashOfHashes, operation)` is called after validators register the bundle in the Header Verifier. It locks the operation hash, mints/unlocks tokens (or performs a contract call), and emits completion. Refunds are handled if execution fails.
- **Token management:** `registerToken` issues a wrapped ESDT for a sovereign token ID (requires fee payment). `registerNativeToken` issues the sovereign chain’s native token during setup. Burn/lock mechanism can be toggled via `setTokenBurnMechanism*` and `setTokenLockMechanism*`.
- **Configuration & safety:** Config updates (`updateEsdtSafeConfig*`), pause control (`pauseContract`), and fee market address are gated by setup checks. `completeSetupPhase` unpauses, ensures native token and fee market are set, and hands off control to the Header Verifier.
- **Interactions:** Owned by the Header Verifier after setup. Calls the Fee Market to subtract fees and the Header Verifier to lock/clear operation hashes.
