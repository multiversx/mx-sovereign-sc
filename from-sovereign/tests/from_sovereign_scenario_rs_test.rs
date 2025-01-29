use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    // blockchain.set_current_dir_from_workspace("relative path to your workspace, if applicable");
    blockchain.register_contract("mxsc:output/from-sovereign.mxsc.json", from_sovereign::ContractBuilder);
    blockchain
}

#[test]
fn empty_rs() {
    world().run("scenarios/from_sovereign.scen.json");
}
