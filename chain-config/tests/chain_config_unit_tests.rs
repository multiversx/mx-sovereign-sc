use multiversx_sc::types::{BigUint, TestAddress, TestSCAddress};
use multiversx_sc_scenario::{
    api::StaticApi, imports::MxscPath, ExpectError, ScenarioTxRun, ScenarioWorld,
};
use proxies::chain_config_proxy::ChainConfigContractProxy;
use operation::SovereignConfig;

const CONFIG_ADDRESS: TestSCAddress = TestSCAddress::new("config-address");
const CONFIG_CODE_PATH: MxscPath = MxscPath::new("output/chain-config.mxsc.json");

const HEADER_VERIFIER_ADDRESS: TestSCAddress = TestSCAddress::new("header-verifier");

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

    fn update_chain_config(
        &mut self,
        config: SovereignConfig<StaticApi>,
        expect_error: Option<ExpectError>,
    ) {
        let transaction = self
            .world
            .tx()
            .from(OWNER)
            .to(CONFIG_ADDRESS)
            .typed(ChainConfigContractProxy)
            .update_config(config);

        if let Some(error) = expect_error {
            transaction.returns(error).run();
        } else {
            transaction.run();
        }
    }

    fn complete_setup_phase(&mut self, expect_error: Option<ExpectError>) {
        let transaction = self
            .world
            .tx()
            .from(OWNER)
            .to(CONFIG_ADDRESS)
            .typed(ChainConfigContractProxy)
            .complete_setup_phase(HEADER_VERIFIER_ADDRESS);

        if let Some(error) = expect_error {
            transaction.returns(error).run();
        } else {
            transaction.run();
        }
    }
}

#[test]
fn deploy_chain_config() {
    let mut state = ChainConfigTestState::new();

    let config = SovereignConfig::new(0, 1, BigUint::default(), None);
    state.deploy_chain_config(config, OWNER);
}

#[test]
fn update_config() {
    let mut state = ChainConfigTestState::new();

    let config = SovereignConfig::new(0, 1, BigUint::default(), None);
    state.deploy_chain_config(config, OWNER);

    let new_config = SovereignConfig::new(2, 4, BigUint::default(), None);

    state.update_chain_config(new_config, None);
}

#[test]
fn update_config_wrong_validators_array() {
    let mut state = ChainConfigTestState::new();

    let config = SovereignConfig::new(0, 1, BigUint::default(), None);
    state.deploy_chain_config(config, OWNER);

    let new_config = SovereignConfig::new(2, 1, BigUint::default(), None);

    state.update_chain_config(
        new_config,
        Some(ExpectError(4, "Invalid min/max validator numbers")),
    );
}

#[test]
fn complete_setup_phase() {
    let mut state = ChainConfigTestState::new();

    let config = SovereignConfig::new(0, 1, BigUint::default(), None);
    state.deploy_chain_config(config, OWNER);

    state.complete_setup_phase(None);
}
