use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract("mxsc:output/execute.mxsc.json", execute::ContractBuilder);
    blockchain
}

#[test]
fn empty_rs() {
    world().run("scenarios/execute.scen.json");
}
