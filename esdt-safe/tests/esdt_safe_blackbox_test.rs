use multiversx_sc::{
    api::StaticVarApi,
    types::{Address, ManagedAddress, ManagedBuffer},
};
use multiversx_sc_scenario::{
    api::StaticApi, scenario_model::{AddressValue, SetStateStep}, ContractInfo, ScenarioWorld,
};

const ESDT_SAFE_PATH_EXPR: &str = "file:output/esdt_safe.wasm";
const INITIATOR_ADDRESS_EXPR: &str = "address:owner";

type ESDTSafeContract = ContractInfo<esdt_safe::Proxy<StaticApi>>;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.set_current_dir_from_workspace("esdt-safe/src/lib");

    blockchain.register_contract(ESDT_SAFE_PATH_EXPR, esdt_safe::ContractBuilder);

    blockchain
}

struct ESDTSafeTestState<M: StaticVarApi> {
    world: ScenarioWorld,
    min_valid_signers: u32,
    initiator_address: Address,
    signers: [Address],
}

impl ESDTSafeTestState {
    fn new() -> Self {
        let mut world = world();

        world.set_state_step(
            SetStateStep::new()
                .put_account(INITIATOR_ADDRESS_EXPR, Account::new().nonce(1))
                .put_account(),
        );

        let initiator_address = AddressValue::from(INITIATOR_ADDRESS_EXPR).to_address();
        let min_valid_signers = 10;

        Self {
            world,
            min_valid_signers,
            initiator_address,
            signers,
        }
    }
}
