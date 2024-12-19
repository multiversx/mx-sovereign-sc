use multiversx_sc::types::{BigUint, MultiValueEncoded, TestAddress, TestSCAddress};
use multiversx_sc_scenario::{api::StaticApi, imports::MxscPath, ScenarioTxRun, ScenarioWorld};
use proxies::chain_config_proxy::ChainConfigContractProxy;
use transaction::{SovereignConfig, StakeArgs};

const CONFIG_ADDRESS: TestSCAddress = TestSCAddress::new("config-address");
const CONFIG_CODE_PATH: MxscPath = MxscPath::new("output/chain-config.mxsc.json");

const OWNER: TestAddress = TestAddress::new("owner");
const OWNER_BALANCE: u64 = 100_000_000_000;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(CONFIG_CODE_PATH, chain_config::ContractBuilder);

    blockchain
}

struct ChainConfigTestState {
    world: ScenarioWorld,
}

impl ChainConfigTestState {
    fn new() -> Self {
        let mut world = world();

        world.account(OWNER).balance(OWNER_BALANCE).nonce(1);

        Self { world }
    }

    fn deploy_chain_config(&mut self, config: SovereignConfig<StaticApi>, admin: TestAddress) {
        self.world
            .tx()
            .from(OWNER)
            .typed(ChainConfigContractProxy)
            .init(config, admin)
            .code(CONFIG_CODE_PATH)
            .new_address(CONFIG_ADDRESS)
            .run();
    }
}

#[test]
fn deploy_chain_config() {
    let mut state = ChainConfigTestState::new();

    let config = SovereignConfig::new(0, 1, BigUint::default(), None);
    state.deploy_chain_config(config, OWNER);
}
