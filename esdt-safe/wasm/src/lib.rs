// Code generated by the multiversx-sc build system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Upgrade:                              1
// Endpoints:                           39
// Async Callback:                       1
// Promise callbacks:                    1
// Total number of exported functions:  43

#![no_std]
#![allow(internal_features)]
#![feature(lang_items)]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    esdt_safe
    (
        init => init
        upgrade => upgrade
        setFeeMarketAddress => set_fee_market_address
        setMultisigAddress => set_header_verifier_address
        setSovereignBridgeAddress => set_sovereign_bridge_address
        setMaxUserTxGasLimit => set_max_user_tx_gas_limit
        setBurnAndMint => set_burn_and_mint
        removeBurnAndMint => remove_burn_and_mint
        addBannedEndpointNames => add_banned_endpoint_names
        removeBannedEndpointNames => remove_banned_endpoint_names
        depositBack => deposit_back
        deposit => deposit
        claimRefund => claim_refund
        setTransactionBatchStatus => set_transaction_batch_status
        setMinValidSigners => set_min_valid_signers
        addSigners => add_signers
        removeSigners => remove_signers
        registerToken => register_token
        clearRegisteredSovereignToken => clear_registered_sovereign_token
        clearRegisteredMultiversxToken => clear_registered_multiversx_token
        executeBridgeOps => execute_operations
        setMaxTxBatchSize => set_max_tx_batch_size
        setMaxTxBatchBlockDuration => set_max_tx_batch_block_duration
        getCurrentTxBatch => get_current_tx_batch
        getFirstBatchAnyStatus => get_first_batch_any_status
        getBatch => get_batch
        getBatchStatus => get_batch_status
        getFirstBatchId => first_batch_id
        getLastBatchId => last_batch_id
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
        execute => execute
    )
}

multiversx_sc_wasm_adapter::async_callback! { esdt_safe }
