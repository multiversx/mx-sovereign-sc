// Code generated by the multiversx-sc build system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Upgrade:                              1
// Endpoints:                            5
// Async Callback (empty):               1
// Total number of exported functions:   8

#![no_std]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    chain_factory
    (
        init => init
        upgrade => upgrade
        deploySovereignChainConfigContract => deploy_sovereign_chain_config_contract
        blacklistSovereignChainSc => blacklist_sovereign_chain_sc
        getDeployCost => deploy_cost
        slash => slash
        distributeSlashed => distribute_slashed
    )
}

multiversx_sc_wasm_adapter::async_callback_empty! {}
