use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(
        "mxsc:output/esdt-safe.mxsc.json",
        esdt_safe::ContractBuilder,
    );
    blockchain
}

#[test]
fn interactor_rs() {
    world().run("interactor/interactor_trace.scen.json");
}
