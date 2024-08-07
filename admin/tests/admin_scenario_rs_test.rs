use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract("mxsc:output/admin.mxsc.json", admin::ContractBuilder);
    blockchain
}

#[test]
fn empty_rs() {
    world().run("scenarios/admin.scen.json");
}
