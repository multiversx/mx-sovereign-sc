// Code generated by the multiversx-sc build system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Upgrade:                              1
// Endpoints:                            0
// Async Callback (empty):               1
// Total number of exported functions:   3

#![no_std]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    liquid_staking
    (
        init => init
        upgrade => upgrade
    )
}

multiversx_sc_wasm_adapter::async_callback_empty! {}
