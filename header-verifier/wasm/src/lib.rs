// Code generated by the multiversx-sc build system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Upgrade:                              1
// Endpoints:                            6
// Async Callback (empty):               1
// Total number of exported functions:   9

#![no_std]
#![allow(internal_features)]
#![feature(lang_items)]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    header_verifier
    (
        init => init
        upgrade => upgrade
        registerBridgeOps => register_bridge_operations
        setEsdtSafeAddress => set_esdt_safe_address
        removeExecutedHash => remove_executed_hash
        setMinValidSigners => set_min_valid_signers
        addSigners => add_signers
        removeSigners => remove_signers
    )
}

multiversx_sc_wasm_adapter::async_callback_empty! {}
