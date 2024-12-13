use std::env::current_dir;

use multiversx_sc::types::{
    BigUint, CodeMetadata, MultiValueEncoded, TestAddress, TestSCAddress, TokenIdentifier,
};
use multiversx_sc_scenario::{
    api::StaticApi, imports::MxscPath, managed_biguint, ExpectError, ScenarioTxRun, ScenarioWorld,
};
use proxies::{
    chain_config_proxy::ChainConfigContractProxy, chain_factory_proxy::ChainFactoryContractProxy,
};
use transaction::StakeArgs;

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
    fn new(additional_stake_required: &MultiValueEncoded<StaticApi, StakeArgs<StaticApi>>) -> Self {
        let mut world = world();

        world.account(OWNER).balance(OWNER_BALANCE).nonce(1);

        // deploy chain-config
        world
            .tx()
            .from(OWNER.to_managed_address())
            .typed(ChainConfigContractProxy)
            .init(
                1usize,
                2usize,
                managed_biguint!(10),
                OWNER.to_managed_address(),
                additional_stake_required,
            )
            .code(CONFIG_CODE_PATH)
            .new_address(CONFIG_ADDRESS)
            .code_metadata(CodeMetadata::UPGRADEABLE)
            .run();

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
        min_validators: usize,
        max_validators: usize,
        min_stake: BigUint<StaticApi>,
        additional_stake_required: MultiValueEncoded<StaticApi, StakeArgs<StaticApi>>,
        expected_result: Option<ExpectError<'_>>,
    ) {
        let transaction = self
            .world
            .tx()
            .from(CONFIG_ADDRESS)
            .to(FACTORY_ADDRESS)
            .typed(ChainFactoryContractProxy)
            .deploy_sovereign_chain_config_contract(
                min_validators,
                max_validators,
                min_stake,
                additional_stake_required,
            );

        match expected_result {
            Some(error) => {
                transaction.returns(error).run();
            }
            None => transaction.run(),
        }
    }
}

#[test]
fn deploy() {
    let additional_stake =
        StakeArgs::new(TokenIdentifier::from("TEST-TOKEN"), BigUint::from(100u64));
    let mut additional_stake_required = MultiValueEncoded::new();
    additional_stake_required.push(additional_stake);

    let mut state = ChainFactoryTestState::new(&additional_stake_required);
    state.deploy_chain_factory();
}

#[test]
fn deploy_chain_config_from_factory() {
    let additional_stake =
        StakeArgs::new(TokenIdentifier::from("TEST-TOKEN"), BigUint::from(100u64));
    let mut additional_stake_required = MultiValueEncoded::new();
    additional_stake_required.push(additional_stake);

    let mut state = ChainFactoryTestState::new(&additional_stake_required);

    let min_validators = 1;
    let max_validators = 4;
    let min_stake = BigUint::from(100_000u64);

    state.deploy_chain_factory();

    println!("{}", current_dir().unwrap().to_str().unwrap());

    state.propose_deploy_chain_config_from_factory(
        min_validators,
        max_validators,
        min_stake,
        additional_stake_required,
        None,
    );
}
