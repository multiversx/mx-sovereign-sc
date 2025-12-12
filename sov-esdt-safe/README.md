# Sovereign ESDT Safe

Sovereign-side vault. It burns incoming assets to emit bridge events, lets admins adjust configuration, and anchors fee collection to the sovereign fee market.

- **Initialization:** `init(fee_market_address, opt_config)` stores the linked fee market, applies config defaults if omitted, and starts paused.
- **Deposits (Sovereign → MultiversX):** `deposit(to, optTransferData)` burns the provided tokens and emits a deposit event containing the transfer data for validators to sign.
- **Registering assets:** `registerToken` issues a sovereign-wrapped token ID (requires the standard issue cost in EGLD). Tokens must carry the sovereign prefix. The call burns the payment and emits a registration event.
- **Configuration:** `updateConfiguration` updates the ESDT Safe config (whitelist/blacklist, gas limits, caps). `setFeeMarketAddress` retargets the fee sink.
- **Interactions:** The Fee Market on the sovereign side is the fee controller. Events emitted here are batched and signed by validators; matching operations are later executed on MultiversX by the Header Verifier + MultiversX ESDT Safe.
