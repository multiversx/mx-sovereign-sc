// Code generated by the multiversx-sc build system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                            0
// Async Callback (empty):               1
// Total number of exported functions:   2

#![no_std]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::external_view_init! {}

multiversx_sc_wasm_adapter::external_view_endpoints! {
    testing_sc
    (
    )
}

multiversx_sc_wasm_adapter::async_callback_empty! {}
