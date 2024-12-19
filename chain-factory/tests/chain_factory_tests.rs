use std::env::current_dir;

use multiversx_sc::types::{BigUint, CodeMetadata, TestAddress, TestSCAddress};
use multiversx_sc_scenario::{
    api::StaticApi, imports::MxscPath, ExpectError, ScenarioTxRun, ScenarioWorld,
};
use proxies::{
    chain_config_proxy::ChainConfigContractProxy, chain_factory_proxy::ChainFactoryContractProxy,
};
use transaction::SovereignConfig;

const FACTORY_ADDRESS: TestSCAddress = TestSCAddress::new("chain-factory");
const CODE_PATH: MxscPath = MxscPath::new("output/chain-factory.mxsc.json");

const CONFIG_ADDRESS: TestSCAddress = TestSCAddress::new("chain-config");
const CONFIG_CODE_PATH: MxscPath = MxscPath::new("../chain-config/output/chain-factory.mxsc.json");

const HEADER_ADDRESS: TestSCAddress = TestSCAddress::new("header-verifier");
const ESDT_SAFE_ADDRESS: TestSCAddress = TestSCAddress::new("esdt-safe");
const FEE_MARKET_ADDRESS: TestSCAddress = TestSCAddress::new("esdt-safe");

const OWNER: TestAddress = TestAddress::new("owner");
const OWNER_BALANCE: u64 = 100_000_000_000;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(CODE_PATH, chain_factory::ContractBuilder);
    blockchain.register_contract(CONFIG_CODE_PATH, chain_config::ContractBuilder);

    blockchain
}

struct ChainFactoryTestState {
    world: ScenarioWorld,
}

impl ChainFactoryTestState {
    fn new() -> Self {
        let mut world = world();

        world.account(OWNER).balance(OWNER_BALANCE).nonce(1);

        // deploy chain-config
        Self { world }
    }

    fn deploy_chain_factory(&mut self) {
        self.world
            .tx()
            .from(OWNER)
            .typed(ChainFactoryContractProxy)
            .init(
                CONFIG_ADDRESS,
                CONFIG_ADDRESS,
                HEADER_ADDRESS,
                ESDT_SAFE_ADDRESS,
                FEE_MARKET_ADDRESS,
            )
            .code(CODE_PATH)
            .new_address(FACTORY_ADDRESS)
            .run();
    }

    fn propose_deploy_chain_config_from_factory(
        &mut self,
        config: SovereignConfig<StaticApi>,
        expected_result: Option<ExpectError<'_>>,
    ) {
        let transaction = self
            .world
            .tx()
            .from(CONFIG_ADDRESS)
            .to(FACTORY_ADDRESS)
            .typed(ChainFactoryContractProxy)
            .deploy_sovereign_chain_config_contract(config);

        match expected_result {
            Some(error) => {
                transaction.returns(error).run();
            }
            None => transaction.run(),
        }
    }

    fn deploy_chain_config(&mut self) {
        let config = SovereignConfig::new(0, 1, BigUint::default(), None);

        self.world
            .tx()
            .from(OWNER.to_managed_address())
            .typed(ChainConfigContractProxy)
            .init(config, OWNER.to_managed_address())
            .code(CONFIG_CODE_PATH)
            .new_address(CONFIG_ADDRESS)
            .code_metadata(CodeMetadata::UPGRADEABLE)
            .run();
    }
}

#[test]
fn deploy() {
    let mut state = ChainFactoryTestState::new();
    state.deploy_chain_factory();
}

#[test]
fn deploy_chain_config_from_factory() {
    let mut state = ChainFactoryTestState::new();
    state.deploy_chain_factory();
    state.deploy_chain_config();

    println!("{}", current_dir().unwrap().to_str().unwrap());

    let config = SovereignConfig::new(0, 1, BigUint::default(), None);

    state.propose_deploy_chain_config_from_factory(config, None);
}
