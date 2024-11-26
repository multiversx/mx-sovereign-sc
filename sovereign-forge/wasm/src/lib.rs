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
    sovereign_forge
    (
        init => init
        upgrade => upgrade
        registerTokenHandler => register_token_handler
        registerChainFactory => register_chain_factory
        completeSetupPhase => complete_setup_phase
        deployPhaseOne => deploy_phase_one
        deployPhaseTwo => deploy_phase_two
        getChainFactoryAddress => chain_factories
        getTokenHandlerAddress => token_handlers
        getDeployCost => deploy_cost
        getAllChainIds => chain_ids
    )
}

multiversx_sc_wasm_adapter::async_callback_empty! {}
