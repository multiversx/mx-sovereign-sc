use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract("mxsc:output/liquid-staking.mxsc.json", liquid_staking::ContractBuilder);
    blockchain
}

#[test]
fn empty_rs() {
    world().run("scenarios/liquid_staking.scen.json");
}
