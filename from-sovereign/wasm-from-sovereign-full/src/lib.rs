// Code generated by the multiversx-sc build system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Upgrade:                              1
// Endpoints:                            8
// Async Callback (empty):               1
// Total number of exported functions:  11

#![no_std]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    from_sovereign
    (
        init => init
        upgrade => upgrade
        setFeeMarketAddress => set_fee_market_address
        deposit => deposit
        registerToken => register_token
        pause => pause_endpoint
        unpause => unpause_endpoint
        isPaused => paused_status
        setMaxBridgedAmount => set_max_bridged_amount
        getMaxBridgedAmount => max_bridged_amount
    )
}

multiversx_sc_wasm_adapter::async_callback_empty! {}
