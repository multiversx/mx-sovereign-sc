use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(
        "mxsc:output/enshrine-esdt-safe.mxsc.json",
        enshrine_esdt_safe::ContractBuilder,
    );
    blockchain
}

#[test]
fn empty_rs() {
    world().run("scenarios/enshrine_esdt_safe.scen.json");
}
