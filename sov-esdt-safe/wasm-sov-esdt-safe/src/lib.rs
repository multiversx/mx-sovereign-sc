// Code generated by the multiversx-sc build system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Upgrade:                              1
// Endpoints:                            9
// Async Callback (empty):               1
// Total number of exported functions:  12

#![no_std]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    sov_esdt_safe
    (
        init => init
        upgrade => upgrade
        updateConfiguration => update_configuration
        setFeeMarketAddress => set_fee_market_address
        setMaxBridgedAmount => set_max_bridged_amount
        deposit => deposit
        getNativeToken => native_token
        getMaxBridgedAmount => max_bridged_amount
        pause => pause_endpoint
        unpause => unpause_endpoint
        isPaused => paused_status
    )
}

multiversx_sc_wasm_adapter::async_callback_empty! {}
