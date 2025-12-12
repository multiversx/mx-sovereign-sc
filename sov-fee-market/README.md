# Sovereign Fee Market

Simple fee controller on the sovereign chain. Owners configure fees, distribute balances, and maintain a whitelist.

- **Initialization:** `init(esdt_safe_address, fee)` links the sovereign ESDT Safe and optionally seeds an initial fee.
- **Operations (owner-only):** `setFee`, `removeFee`, `distributeFees`, `addUsersToWhitelist`, `removeUsersFromWhitelist`.
- **Interactions:** The sovereign ESDT Safe references this contract for fee lookups when processing deposits destined for MultiversX.
