// Code generated by the multiversx-sc build system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Upgrade:                              1
// Endpoints:                           19
// Async Callback:                       1
// Total number of exported functions:  22

#![no_std]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    enshrine_esdt_safe
    (
        init => init
        upgrade => upgrade
        setFeeMarketAddress => set_fee_market_address
        setHeaderVerifierAddress => set_header_verifier_address
        setMaxTxGasLimit => set_max_user_tx_gas_limit
        setBannedEndpoint => set_banned_endpoint
        deposit => deposit
        executeBridgeOps => execute_operations
        registerNewTokenID => register_new_token_id
        setMaxBridgedAmount => set_max_bridged_amount
        getMaxBridgedAmount => max_bridged_amount
        endSetupPhase => end_setup_phase
        addTokensToWhitelist => add_tokens_to_whitelist
        removeTokensFromWhitelist => remove_tokens_from_whitelist
        addTokensToBlacklist => add_tokens_to_blacklist
        removeTokensFromBlacklist => remove_tokens_from_blacklist
        getTokenWhitelist => token_whitelist
        getTokenBlacklist => token_blacklist
        pause => pause_endpoint
        unpause => unpause_endpoint
        isPaused => paused_status
    )
}

multiversx_sc_wasm_adapter::async_callback! { enshrine_esdt_safe }
