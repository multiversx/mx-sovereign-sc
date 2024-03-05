
use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();
    blockchain.set_current_dir_from_workspace("contracts/multisig");

    blockchain.register_partial_contract::<multisigverifier::AbiProvider, _>(
        "file:output/multisig.wasm",
        multisigverifier::ContractBuilder,
        "multisig",
    );
    blockchain.register_partial_contract::<multisigverifier::AbiProvider, _>(
        "file:output/multisig-view.wasm",
        multisigverifier::ContractBuilder,
        "multisig-view",
    );

    blockchain
}
