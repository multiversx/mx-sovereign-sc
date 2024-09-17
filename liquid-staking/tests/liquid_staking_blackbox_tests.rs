use liquid_staking::{common::storage::CommonStorageModule, liquid_staking_proxy};
use multiversx_sc::types::{BigUint, ManagedBuffer, TestAddress, TestSCAddress};
use multiversx_sc_scenario::{
    api::StaticApi, imports::MxscPath, DebugApi, ExpectError, ScenarioTxRun, ScenarioTxWhitebox,
    ScenarioWorld,
};

const LIQUID_STAKING_CODE_PATH: MxscPath = MxscPath::new("output/liquid-stacking.mxsc-json");
const LIQUID_STAKING_ADDRESS: TestSCAddress = TestSCAddress::new("liquid-staking");
const LIQUID_STACKING_OWNER: TestAddress = TestAddress::new("owner");

const DELEGATION_ADDRESS: TestSCAddress = TestSCAddress::new("delegation");

const VALIDATOR_ADDRESS: TestAddress = TestAddress::new("validator");

const WEGLD_BALANCE: u128 = 100_000_000_000_000_000;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(LIQUID_STAKING_CODE_PATH, liquid_staking::ContractBuilder);

    blockchain
}

struct LiquidStakingTestState {
    world: ScenarioWorld,
}

pub struct ErrorStatus<'a> {
    code: u64,
    error_message: &'a str,
}

impl LiquidStakingTestState {
    fn new() -> Self {
        let mut world = world();

        world
            .account(LIQUID_STACKING_OWNER)
            .balance(BigUint::from(WEGLD_BALANCE))
            .nonce(1);

        world
            .account(VALIDATOR_ADDRESS)
            .balance(BigUint::from(WEGLD_BALANCE))
            .nonce(1);

        Self { world }
    }

    fn deploy_liquid_staking(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(LIQUID_STACKING_OWNER)
            .typed(liquid_staking_proxy::LiquidStakingProxy)
            .init()
            .code(LIQUID_STAKING_CODE_PATH)
            .new_address(LIQUID_STAKING_ADDRESS)
            .run();

        self
    }

    fn propose_register_delegation_address(
        &mut self,
        contract_name: &ManagedBuffer<StaticApi>,
        delegation_address: TestSCAddress,
        error_status: Option<ErrorStatus>,
    ) -> &mut Self {
        match error_status {
            Some(error) => self
                .world
                .tx()
                .from(LIQUID_STACKING_OWNER)
                .to(LIQUID_STAKING_ADDRESS)
                .typed(liquid_staking_proxy::LiquidStakingProxy)
                .register_delegation_address(contract_name, delegation_address)
                .returns(ExpectError(error.code, error.error_message))
                .run(),
            None => self
                .world
                .tx()
                .from(LIQUID_STACKING_OWNER)
                .to(LIQUID_STAKING_ADDRESS)
                .typed(liquid_staking_proxy::LiquidStakingProxy)
                .register_delegation_address(contract_name, delegation_address)
                .run(),
        }
        self
    }

    fn propose_stake(
        &mut self,
        contract_name: &ManagedBuffer<StaticApi>,
        payment: &BigUint<StaticApi>,
    ) -> &mut Self {
        self.world
            .tx()
            .from(LIQUID_STACKING_OWNER)
            .to(LIQUID_STAKING_ADDRESS)
            .typed(liquid_staking_proxy::LiquidStakingProxy)
            .stake(contract_name)
            .egld(payment)
            .run();

        self
    }
}

#[test]
fn test_deploy() {
    let mut state = LiquidStakingTestState::new();

    state.deploy_liquid_staking();
}

#[test]
fn test_register_delegation_contract() {
    let mut state = LiquidStakingTestState::new();
    let contract_name = ManagedBuffer::from("delegation");

    state.deploy_liquid_staking();
    state.propose_register_delegation_address(&contract_name, DELEGATION_ADDRESS, None);

    state
        .world
        .query()
        .to(LIQUID_STAKING_ADDRESS)
        .whitebox(liquid_staking::contract_obj, |sc| {
            let contract_name_debug_api: ManagedBuffer<DebugApi> =
                ManagedBuffer::from("delegation");
            let registered_address = sc.delegation_addresses(&contract_name_debug_api).get();
            assert_eq!(DELEGATION_ADDRESS, registered_address);
        });
}

#[test]
fn test_register_delegation_contract_contract_already_registered() {
    let mut state = LiquidStakingTestState::new();
    let error_status = ErrorStatus {
        code: 4,
        error_message: "This contract is already registered",
    };

    state.deploy_liquid_staking();
    state.propose_register_delegation_address(&ManagedBuffer::new(), DELEGATION_ADDRESS, None);
    state.propose_register_delegation_address(
        &ManagedBuffer::new(),
        DELEGATION_ADDRESS,
        Some(error_status),
    );
}

#[test]
fn test_stake() {
    let mut state = LiquidStakingTestState::new();
    let contract_name = ManagedBuffer::from("delegation");
    let payment = BigUint::from(100_000u64);

    state.deploy_liquid_staking();
    state.propose_register_delegation_address(&contract_name, DELEGATION_ADDRESS, None);
    state.propose_stake(&contract_name, &payment);

    state
        .world
        .check_account(LIQUID_STAKING_ADDRESS)
        .balance(BigUint::from(WEGLD_BALANCE) - payment);
}
