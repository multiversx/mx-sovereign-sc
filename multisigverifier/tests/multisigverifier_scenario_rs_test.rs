use multiversx_sc_scenario::*;
mod multisigverifier_setup;
fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();
    // blockchain.set_current_dir_from_workspace("relative path to your workspace, if applicable");

    blockchain.register_contract("file:output/multisigverifier.wasm", multisigverifier::ContractBuilder);
    blockchain
}

#[test]
fn empty_rs() {
    world().run("scenarios/multisigverifier.scen.json");
}
