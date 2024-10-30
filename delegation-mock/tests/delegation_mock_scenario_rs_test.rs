use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract("mxsc:output/delegation-mock.mxsc.json", delegation_mock::ContractBuilder);
    blockchain
}

#[test]
fn empty_rs() {
    world().run("scenarios/delegation_mock.scen.json");
}
