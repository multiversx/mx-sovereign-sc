// Code generated by the multiversx-sc multi-contract system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                           30
// Async Callback:                       1
// Promise callbacks:                    1
// Total number of exported functions:  33

#![no_std]

// Configuration that works with rustc < 1.73.0.
// TODO: Recommended rustc version: 1.73.0 or newer.
#![feature(lang_items)]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    esdt_safe
    (
        init => init
        upgrade => upgrade
        createTransaction => create_transaction
        getSovereignTxGasLimit => sovereign_tx_gas_limit
        addRefundBatch => add_refund_batch
        claimRefund => claim_refund
        getRefundAmounts => get_refund_amounts
        setTransactionBatchStatus => set_transaction_batch_status
        setMinValidSigners => set_min_valid_signers
        addSigners => add_signers
        removeSigners => remove_signers
        getAndClearFirstRefundBatch => get_and_clear_first_refund_batch
        registerToken => register_token
        clearRegisteredToken => clear_registered_token
        batchTransferEsdtToken => batch_transfer_esdt_token
        addTokenToWhitelist => add_token_to_whitelist
        removeTokenFromWhitelist => remove_token_from_whitelist
        getAllKnownTokens => token_whitelist
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
        pause => pause_endpoint
        unpause => unpause_endpoint
        isPaused => paused_status
        transfer_callback => transfer_callback
    )
}

multiversx_sc_wasm_adapter::async_callback! { esdt_safe }
