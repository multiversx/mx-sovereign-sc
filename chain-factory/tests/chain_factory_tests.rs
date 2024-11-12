use std::env::current_dir;

use chain_config::{chain_config_proxy, StakeMultiArg};
use chain_factory::{
    chain_factory_proxy::{self, ContractInfo, ScArray},
    common::storage::CommonStorage,
};
use multiversx_sc::types::{
    BigUint, CodeMetadata, ManagedBuffer, MultiValueEncoded, TestAddress, TestSCAddress,
    TokenIdentifier,
};
use multiversx_sc_scenario::{
    api::StaticApi, imports::MxscPath, managed_biguint, ExpectError, ScenarioTxRun,
    ScenarioTxWhitebox, ScenarioWorld,
};

const FACTORY_ADDRESS: TestSCAddress = TestSCAddress::new("chain-factory");
const CODE_PATH: MxscPath = MxscPath::new("output/chain-factory.mxsc.json");
const CHAIN_ID: &str = "CHAIN-ID";

const CONFIG_ADDRESS: TestSCAddress = TestSCAddress::new("chain-config");
const CONFIG_CODE_PATH: MxscPath = MxscPath::new("../chain-config/output/chain-factory.mxsc.json");

const OWNER: TestAddress = TestAddress::new("owner");
const OWNER_BALANCE: u64 = 100_000_000_000;
const DEPLOY_COST: u64 = 10_000_000_000;

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
    fn new(
        additional_stake_required: &MultiValueEncoded<StaticApi, StakeMultiArg<StaticApi>>,
    ) -> Self {
        let mut world = world();

        world.account(OWNER).balance(OWNER_BALANCE).nonce(1);

        // deploy chain-config
        world
            .tx()
            .from(OWNER.to_managed_address())
            .typed(chain_config_proxy::ChainConfigContractProxy)
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

    fn propose_add_contracts_to_map(
        &mut self,
        chain_id: ManagedBuffer<StaticApi>,
        contracts_info: MultiValueEncoded<StaticApi, ContractInfo<StaticApi>>,
    ) {
        self.world
            .tx()
            .from(OWNER)
            .to(FACTORY_ADDRESS)
            .typed(chain_factory_proxy::ChainFactoryContractProxy)
            .add_contracts_to_map(chain_id, contracts_info)
            .run();
    }

    fn propose_deploy_chain_config_from_factory(
        &mut self,
        payment: BigUint<StaticApi>,
        min_validators: usize,
        max_validators: usize,
        min_stake: BigUint<StaticApi>,
        additional_stake_required: MultiValueEncoded<StaticApi, StakeMultiArg<StaticApi>>,
        expected_result: Option<ExpectError<'_>>,
    ) {
        let transaction = self
            .world
            .tx()
            .from(OWNER)
            .to(FACTORY_ADDRESS)
            .typed(chain_factory_proxy::ChainFactoryContractProxy)
            .deploy_sovereign_chain_config_contract(
                min_validators,
                max_validators,
                min_stake,
                additional_stake_required,
            )
            .egld(payment);

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
    let additional_stake: StakeMultiArg<StaticApi> =
        (TokenIdentifier::from("TEST-TOKEN"), BigUint::from(100u64)).into();
    let mut additional_stake_required = MultiValueEncoded::new();
    additional_stake_required.push(additional_stake);

    let mut state = ChainFactoryTestState::new(&additional_stake_required);
    state.deploy_chain_factory();
}

#[test]
fn add_contracts_to_map() {
    let additional_stake: StakeMultiArg<StaticApi> =
        (TokenIdentifier::from("TEST-TOKEN"), BigUint::from(100u64)).into();
    let mut additional_stake_required = MultiValueEncoded::new();
    additional_stake_required.push(additional_stake);

    let mut state = ChainFactoryTestState::new(&additional_stake_required);
    state.deploy_chain_factory();

    let contract_info: ContractInfo<StaticApi> = ContractInfo {
        id: ScArray::ChainConfig,
        address: CONFIG_ADDRESS.to_managed_address(),
    };
    let mut contracts_map = MultiValueEncoded::new();
    contracts_map.push(contract_info);

    state.propose_add_contracts_to_map(ManagedBuffer::from(CHAIN_ID), contracts_map);

    state
        .world
        .query()
        .to(FACTORY_ADDRESS)
        .whitebox(chain_factory::contract_obj, |sc| {
            assert!(!sc
                .all_deployed_contracts(&OWNER.to_managed_address())
                .is_empty());
        })
}

#[test]
fn deploy_chain_config_from_factory_deploy_cost_too_low() {
    let additional_stake: StakeMultiArg<StaticApi> =
        (TokenIdentifier::from("TEST-TOKEN"), BigUint::from(100u64)).into();
    let mut additional_stake_required = MultiValueEncoded::new();
    additional_stake_required.push(additional_stake);

    let mut state = ChainFactoryTestState::new(&additional_stake_required);

    let min_validators = 1;
    let max_validators = 4;
    let min_stake = BigUint::from(100_000u64);

    state.deploy_chain_factory();

    state.propose_deploy_chain_config_from_factory(
        BigUint::from(100u64),
        min_validators,
        max_validators,
        min_stake,
        additional_stake_required,
        Some(ExpectError(4, "Invalid payment amount")),
    );
}

#[test]
fn deploy_chain_config_from_factory() {
    let additional_stake: StakeMultiArg<StaticApi> =
        (TokenIdentifier::from("TEST-TOKEN"), BigUint::from(100u64)).into();
    let mut additional_stake_required = MultiValueEncoded::new();
    additional_stake_required.push(additional_stake);

    let mut state = ChainFactoryTestState::new(&additional_stake_required);

    let min_validators = 1;
    let max_validators = 4;
    let min_stake = BigUint::from(100_000u64);

    state.deploy_chain_factory();

    println!("{}", current_dir().unwrap().to_str().unwrap());

    state.propose_deploy_chain_config_from_factory(
        DEPLOY_COST.into(),
        min_validators,
        max_validators,
        min_stake,
        additional_stake_required,
        None,
    );

    state
        .world
        .query()
        .to(FACTORY_ADDRESS)
        .whitebox(chain_factory::contract_obj, |sc| {
            assert!(!sc
                .all_deployed_contracts(&OWNER.to_managed_address())
                .is_empty());
        })
}
