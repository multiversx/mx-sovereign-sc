use chain_config::{chain_config_proxy, StakeMultiArg};
use chain_factory::chain_factory_proxy::{self, ContractMapArgs};
use multiversx_sc::types::{
    BigUint, MultiValueEncoded, TestAddress, TestSCAddress, TokenIdentifier,
};
use multiversx_sc_scenario::{
    api::StaticApi, imports::MxscPath, managed_biguint, ScenarioTxRun, ScenarioWorld,
};
use num_bigint::BigUint;

const FACTORY_ADDRESS: TestSCAddress = TestSCAddress::new("chain-factory");
const CODE_PATH: MxscPath = MxscPath::new("output/chain-factory.mxsc.json");

const CONFIG_ADDRESS: TestSCAddress = TestSCAddress::new("chain-config");
const CONFIG_CODE_PATH: MxscPath = MxscPath::new("../chain-config/output/chain-factory.mxsc.json");

const OWNER: TestAddress = TestAddress::new("owner");
const DEPLOY_COST: u64 = 100_000_000_000;

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

        world.account(OWNER).balance(100_000).nonce(1);

        Self { world }
    }

    fn deploy_chain_factory(&mut self) {
        self.world
            .tx()
            .from(OWNER)
            .typed(chain_factory_proxy::ChainFactoryContractProxy)
            .init(
                FACTORY_ADDRESS,
                CONFIG_ADDRESS,
                FACTORY_ADDRESS,
                FACTORY_ADDRESS,
                FACTORY_ADDRESS,
                FACTORY_ADDRESS,
                managed_biguint!(DEPLOY_COST),
            )
            .code(CODE_PATH)
            .new_address(FACTORY_ADDRESS)
            .run();
    }

    fn deploy_chain_config(
        &mut self,
        min_validators: usize,
        max_validators: usize,
        min_stake: BigUint,
        admin: TestAddress,
        additional_stake_required: MultiValueEncoded<StaticApi, StakeMultiArg<StaticApi>>,
    ) {
        self.world
            .tx()
            .from(OWNER)
            .typed(chain_config_proxy::ChainConfigContractProxy)
            .init(
                min_validators,
                max_validators,
                min_stake,
                admin,
                additional_stake_required,
            )
            .code(CONFIG_CODE_PATH)
            .new_address(CONFIG_ADDRESS)
            .run();
    }

    fn propose_add_contracts_to_map(
        &mut self,
        contracts_map: MultiValueEncoded<StaticApi, ContractMapArgs<StaticApi>>,
    ) {
        self.world
            .tx()
            .from(OWNER)
            .to(FACTORY_ADDRESS)
            .typed(chain_factory_proxy::ChainFactoryContractProxy)
            .add_contracts_to_map(contracts_map)
            .run();
    }
}

#[test]
fn deploy_test() {
    let mut state = ChainFactoryTestState::new();
    state.deploy_chain_factory();

    let min_validators = 1;
    let max_validators = 4;
    let min_stake = BigUint::from(100_000u64);
    let additional_stake = vec![(TokenIdentifier::from("TEST-TOKEN"), BigUint::from(100u64))];
    let additional_stake_required = MultiValueEncoded::new();

    state.deploy_chain_config(
        min_validators,
        max_validators,
        min_stake,
        OWNER,
        additional_stake_required,
    );
}

#[test]
fn add_contracts_to_map_test() {
    let mut state = ChainFactoryTestState::new();
    state.deploy_chain_factory();
}
