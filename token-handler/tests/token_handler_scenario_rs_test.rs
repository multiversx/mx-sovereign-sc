use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(
        "mxsc:output/token-handler.mxsc.json",
        token_handler::ContractBuilder,
    );
    blockchain
}
