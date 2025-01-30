use multiversx_sc::types::{TestAddress, TestSCAddress};
use multiversx_sc_scenario::{api::StaticApi, imports::MxscPath, ScenarioTxRun, ScenarioWorld};
use operation::CrossChainConfig;
use proxies::from_sovereign_proxy::FromSovereignProxy;

const CONTRACT_ADDRESS: TestSCAddress = TestSCAddress::new("sc");
const CONTRACT_CODE_PATH: MxscPath = MxscPath::new("output/from-sovereign.mxsc.json");
const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");

const OWNER_BALANCE: u64 = 100_000_000;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(CONTRACT_CODE_PATH, from_sovereign::ContractBuilder);

    blockchain
}
struct FromSovereignTestState {
    world: ScenarioWorld,
}

impl FromSovereignTestState {
    fn new() -> Self {
        let mut world = world();

        world.account(OWNER_ADDRESS).nonce(1).balance(OWNER_BALANCE);

        Self { world }
    }

    fn deploy_contract(&mut self, config: CrossChainConfig<StaticApi>) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(FromSovereignProxy)
            .init(config)
            .code(CONTRACT_CODE_PATH)
            .new_address(CONTRACT_ADDRESS)
            .run();

        self
    }
}

#[test]
fn deploy() {
    let mut state = FromSovereignTestState::new();

    state.deploy_contract(CrossChainConfig::default_config());
}
