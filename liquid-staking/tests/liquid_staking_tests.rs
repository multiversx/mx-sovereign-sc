use delegation_mock::{APY, EPOCHS_IN_YEAR, MAX_PERCENTAGE};
use liquid_staking::{
    common::storage::CommonStorageModule, delegation_proxy, liquid_staking_proxy,
};
use multiversx_sc::types::{
    BigUint, ManagedAddress, ManagedBuffer, MultiValueEncoded, TestAddress, TestSCAddress,
};
use multiversx_sc_scenario::{
    api::StaticApi, imports::MxscPath, DebugApi, ExpectError, ScenarioTxRun, ScenarioTxWhitebox,
    ScenarioWorld,
};

const LIQUID_STAKING_CODE_PATH: MxscPath = MxscPath::new("output/liquid-stacking.mxsc.json");
const LIQUID_STAKING_ADDRESS: TestSCAddress = TestSCAddress::new("liquid-staking");
const OWNER: TestAddress = TestAddress::new("owner");

const DELEGATION_CODE_PATH: MxscPath =
    MxscPath::new("../delegation-mock/output/delegation-mock.mxsc.json");
const DELEGATION_ADDRESS: TestSCAddress = TestSCAddress::new("delegation");

const MOCK_CODE_PATH: MxscPath =
    MxscPath::new("../header-verifier/output/header-verifier.mxsc.json");

const HEADER_VERIFIER_ADDRESS: TestSCAddress = TestSCAddress::new("header_verifier");

const VALIDATOR_ADDRESS: TestAddress = TestAddress::new("validator");

const WEGLD_BALANCE: u128 = 100_000_000_000_000_000;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(LIQUID_STAKING_CODE_PATH, liquid_staking::ContractBuilder);
    blockchain.register_contract(DELEGATION_CODE_PATH, delegation_mock::ContractBuilder);
    blockchain.register_contract(MOCK_CODE_PATH, delegation_mock::ContractBuilder);

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
            .account(OWNER)
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
            .from(OWNER)
            .typed(liquid_staking_proxy::LiquidStakingProxy)
            .init()
            .code(LIQUID_STAKING_CODE_PATH)
            .new_address(LIQUID_STAKING_ADDRESS)
            .run();

        self
    }

    fn deploy_delegation(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER)
            .typed(delegation_proxy::DelegationMockProxy)
            .init()
            .code(DELEGATION_CODE_PATH)
            .new_address(DELEGATION_ADDRESS)
            .run();

        self
    }

    fn deploy_mock_header_verifier(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER)
            .typed(delegation_proxy::DelegationMockProxy)
            .init()
            .code(MOCK_CODE_PATH)
            .new_address(HEADER_VERIFIER_ADDRESS)
            .run();

        self
    }

    fn propose_setup_contracts(&mut self) -> &mut Self {
        self.deploy_liquid_staking();
        self.deploy_delegation();
        self.deploy_mock_header_verifier();

        self
    }

    fn propose_register_delegation_address(
        &mut self,
        contract_name: &ManagedBuffer<StaticApi>,
        delegation_address: TestSCAddress,
        error_status: Option<ErrorStatus>,
    ) -> &mut Self {
        let managed_delegation_address = delegation_address.to_managed_address();
        match error_status {
            Some(error) => self
                .world
                .tx()
                .from(OWNER)
                .to(LIQUID_STAKING_ADDRESS)
                .typed(liquid_staking_proxy::LiquidStakingProxy)
                .register_delegation_address(contract_name, managed_delegation_address)
                .returns(ExpectError(error.code, error.error_message))
                .run(),
            None => self
                .world
                .tx()
                .from(OWNER)
                .to(LIQUID_STAKING_ADDRESS)
                .typed(liquid_staking_proxy::LiquidStakingProxy)
                .register_delegation_address(contract_name, managed_delegation_address)
                .run(),
        }
        self
    }

    fn propose_stake(
        &mut self,
        validator: &TestAddress,
        contract_name: &ManagedBuffer<StaticApi>,
        payment: &BigUint<StaticApi>,
    ) -> &mut Self {
        self.world
            .tx()
            .from(validator.to_managed_address())
            .to(LIQUID_STAKING_ADDRESS)
            .typed(liquid_staking_proxy::LiquidStakingProxy)
            .stake(contract_name)
            .egld(payment)
            .run();

        self
    }

    fn propose_unstake(
        &mut self,
        contract_name: &ManagedBuffer<StaticApi>,
        amount_to_unstake: &BigUint<StaticApi>,
        error_status: Option<ErrorStatus>,
    ) -> &mut Self {
        match error_status {
            Some(error) => self
                .world
                .tx()
                .from(OWNER)
                .to(LIQUID_STAKING_ADDRESS)
                .typed(liquid_staking_proxy::LiquidStakingProxy)
                .unstake(contract_name, amount_to_unstake)
                .returns(ExpectError(error.code, error.error_message))
                .run(),
            None => self
                .world
                .tx()
                .from(OWNER)
                .to(LIQUID_STAKING_ADDRESS)
                .typed(liquid_staking_proxy::LiquidStakingProxy)
                .unstake(contract_name, amount_to_unstake)
                .run(),
        }

        self
    }

    fn propose_claim_rewards_from_delegation(
        &mut self,
        contracts: &MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>>,
    ) -> &mut Self {
        self.world
            .tx()
            .from(OWNER)
            .to(LIQUID_STAKING_ADDRESS)
            .typed(liquid_staking_proxy::LiquidStakingProxy)
            .claim_rewards_from_delegation(contracts)
            .run();

        self
    }

    fn propose_register_bls_keys(
        &mut self,
        bls_keys: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>>,
        error_status: Option<ErrorStatus>,
    ) {
        match error_status {
            Some(error) => self
                .world
                .tx()
                .from(OWNER)
                .to(LIQUID_STAKING_ADDRESS)
                .typed(liquid_staking_proxy::LiquidStakingProxy)
                .register_bls_keys(bls_keys)
                .returns(ExpectError(error.code, error.error_message))
                .run(),
            None => self
                .world
                .tx()
                .from(HEADER_VERIFIER_ADDRESS)
                .to(LIQUID_STAKING_ADDRESS)
                .typed(liquid_staking_proxy::LiquidStakingProxy)
                .register_bls_keys(bls_keys)
                .run(),
        }
    }

    fn propose_register_header_verifier(
        &mut self,
        header_verifier_address: TestSCAddress,
    ) -> &mut Self {
        self.world
            .tx()
            .from(OWNER)
            .to(LIQUID_STAKING_ADDRESS)
            .typed(liquid_staking_proxy::LiquidStakingProxy)
            .register_header_verifier_address(header_verifier_address)
            .run();

        self
    }

    fn propose_slash_validator(
        &mut self,
        bls_key: &ManagedBuffer<StaticApi>,
        value_to_slash: BigUint<StaticApi>,
        error_status: Option<ErrorStatus>,
    ) {
        match error_status {
            Some(error) => self
                .world
                .tx()
                .from(HEADER_VERIFIER_ADDRESS)
                .to(LIQUID_STAKING_ADDRESS)
                .typed(liquid_staking_proxy::LiquidStakingProxy)
                .slash_validator(bls_key, value_to_slash)
                .returns(ExpectError(error.code, error.error_message))
                .run(),
            None => self
                .world
                .tx()
                .from(HEADER_VERIFIER_ADDRESS)
                .to(LIQUID_STAKING_ADDRESS)
                .typed(liquid_staking_proxy::LiquidStakingProxy)
                .slash_validator(bls_key, value_to_slash)
                .run(),
        }
    }

    fn propose_check_is_contract_registered(&mut self, contract_name: &ManagedBuffer<StaticApi>) {
        self.world.query().to(LIQUID_STAKING_ADDRESS).whitebox(
            liquid_staking::contract_obj,
            |sc| {
                let contract_name_debug_api: ManagedBuffer<DebugApi> =
                    ManagedBuffer::from(contract_name.to_vec());
                let registered_address = sc.delegation_addresses(&contract_name_debug_api).get();
                assert_eq!(DELEGATION_ADDRESS, registered_address);
            },
        );
    }

    fn get_expected_rewards(
        &mut self,
        staked_amount: &BigUint<StaticApi>,
        current_epoch: u64,
        last_claim_epoch: u64,
    ) -> BigUint<StaticApi> {
        let rewards = (staked_amount * APY / MAX_PERCENTAGE) * (current_epoch - last_claim_epoch)
            / EPOCHS_IN_YEAR;

        rewards
    }

    fn map_bls_key_vec_to_multi_value(
        &mut self,
        bls_keys: Vec<&ManagedBuffer<StaticApi>>,
    ) -> MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>> {
        let managed_bls_keys: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>> =
            bls_keys.iter().map(|bls_key| (*bls_key).clone()).collect();

        managed_bls_keys
    }

    fn whitebox_map_bls_to_address(&mut self, bls_key: &str, address: &TestAddress) -> &mut Self {
        self.world
            .tx()
            .from(OWNER)
            .to(LIQUID_STAKING_ADDRESS)
            .whitebox(liquid_staking::contract_obj, |sc| {
                let validator_address: ManagedAddress<DebugApi> = address.to_managed_address();
                let bls_key_whitebox: ManagedBuffer<DebugApi> = ManagedBuffer::from(bls_key);
                sc.validator_bls_key_address_map(&bls_key_whitebox)
                    .set(validator_address);
            });

        self
    }
}

#[test]
fn test_deploy() {
    let mut state = LiquidStakingTestState::new();

    state.propose_setup_contracts();
}

#[test]
fn test_register_delegation_contract() {
    let mut state = LiquidStakingTestState::new();
    let contract_name = ManagedBuffer::from("delegation");

    state.propose_setup_contracts();
    state.propose_register_delegation_address(&contract_name, DELEGATION_ADDRESS, None);
    state.propose_check_is_contract_registered(&contract_name);
}

#[test]
fn test_register_delegation_contract_contract_already_registered() {
    let mut state = LiquidStakingTestState::new();
    let error_status = ErrorStatus {
        code: 4,
        error_message: "This contract is already registered",
    };

    state.propose_setup_contracts();
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
    let contract_name = ManagedBuffer::from("delegation_sc");
    let payment = BigUint::from(100_000u64);

    state.propose_setup_contracts();
    state.propose_register_delegation_address(&contract_name, DELEGATION_ADDRESS, None);
    state.propose_check_is_contract_registered(&contract_name);
    state.propose_stake(&OWNER, &contract_name, &payment);

    state
        .world
        .check_account(OWNER)
        .balance(BigUint::from(WEGLD_BALANCE) - payment);

    state
        .world
        .query()
        .to(LIQUID_STAKING_ADDRESS)
        .whitebox(liquid_staking::contract_obj, |sc| {
            let payment_whitebox = BigUint::from(100_000u64);
            let delegated_value = sc.delegated_value(&OWNER.to_managed_address()).get();
            let egld_supply = sc.egld_token_supply().get();

            assert!(egld_supply > 0);
            assert_eq!(delegated_value, payment_whitebox);
        });
}

#[test]
fn test_unstake_user_no_deposit() {
    let mut state = LiquidStakingTestState::new();
    let contract_name = ManagedBuffer::from("delegation");
    let payment = BigUint::from(100_000u64);
    let error_status = ErrorStatus {
        code: 4,
        error_message: "Caller has 0 delegated value",
    };

    state.propose_setup_contracts();
    state.propose_register_delegation_address(&contract_name, DELEGATION_ADDRESS, None);
    state.propose_check_is_contract_registered(&contract_name);
    state.propose_unstake(&contract_name, &payment, Some(error_status));
}

#[test]
fn test_unstake() {
    let mut state = LiquidStakingTestState::new();
    let contract_name = ManagedBuffer::from("delegation");
    let payment = BigUint::from(100_000u64);
    let amount_to_unstake = BigUint::from(10_000u64);

    state.propose_setup_contracts();
    state.propose_register_delegation_address(&contract_name, DELEGATION_ADDRESS, None);
    state.propose_check_is_contract_registered(&contract_name);
    state.propose_stake(&OWNER, &contract_name, &payment);
    state.propose_unstake(&contract_name, &amount_to_unstake, None);

    state
        .world
        .check_account(OWNER)
        .balance(BigUint::from(WEGLD_BALANCE) - payment);

    state
        .world
        .query()
        .to(LIQUID_STAKING_ADDRESS)
        .whitebox(liquid_staking::contract_obj, |sc| {
            let expected_amount = BigUint::from(90_000u64);
            let delegated_value = sc.delegated_value(&OWNER.to_managed_address()).get();

            assert_eq!(delegated_value, expected_amount);
        })
}

#[test]
fn test_claim_rewards_from_delegation_contract() {
    let mut state = LiquidStakingTestState::new();
    let contract_name = ManagedBuffer::from("delegation");
    let payment = BigUint::from(100_000u64);
    let mut contracts_to_claim_from: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>> =
        MultiValueEncoded::new();
    let claim_rewards_epoch = 20;

    state.propose_setup_contracts();
    state.propose_register_delegation_address(&contract_name, DELEGATION_ADDRESS, None);
    state.propose_check_is_contract_registered(&contract_name);
    state.propose_stake(&OWNER, &contract_name, &payment);

    contracts_to_claim_from.push(contract_name);

    state.world.current_block().block_epoch(claim_rewards_epoch);

    state.propose_claim_rewards_from_delegation(&contracts_to_claim_from);
    state.world.current_block().block_epoch(claim_rewards_epoch);
    state.propose_claim_rewards_from_delegation(&contracts_to_claim_from);

    let expected_rewards = state.get_expected_rewards(&payment, claim_rewards_epoch, 0);

    state
        .world
        .check_account(LIQUID_STAKING_ADDRESS)
        .balance(expected_rewards);
}

#[test]
fn test_slash_validator() {
    let mut state = LiquidStakingTestState::new();
    let contract_name = ManagedBuffer::from("delegation");
    let payment = BigUint::from(100_000u64);
    // let validator_1_bls_key = ManagedBuffer::from("validator1");
    // let validator_2_bls_key = ManagedBuffer::from("validator2");

    state.propose_setup_contracts();
    state.propose_register_delegation_address(&contract_name, DELEGATION_ADDRESS, None);
    state.propose_check_is_contract_registered(&contract_name);
    state.propose_stake(&OWNER, &contract_name, &payment);
    // state.propose_slash_validator(bls_key, value_to_slash);
}

#[test]
fn test_register_bls_keys_no_header_verifier_address() {
    let mut state = LiquidStakingTestState::new();
    let contract_name = ManagedBuffer::from("delegation");
    let bls_keys = state.map_bls_key_vec_to_multi_value(vec![&ManagedBuffer::from("bls_key")]);
    let error_status = ErrorStatus {
        code: 4,
        error_message: "There is no address registered as the Header Verifier",
    };

    state.propose_setup_contracts();
    state.propose_register_delegation_address(&contract_name, DELEGATION_ADDRESS, None);
    state.propose_check_is_contract_registered(&contract_name);
    state.propose_register_bls_keys(bls_keys, Some(error_status));
}

#[test]
fn test_register_bls_keys_caller_not_header_verifier() {
    let mut state = LiquidStakingTestState::new();
    let contract_name = ManagedBuffer::from("delegation");
    let bls_keys = state.map_bls_key_vec_to_multi_value(vec![&ManagedBuffer::from("bls_key")]);
    let error_status = ErrorStatus {
        code: 4,
        error_message: "Caller is not Header Verifier contract",
    };

    state.propose_setup_contracts();
    state.propose_register_delegation_address(&contract_name, DELEGATION_ADDRESS, None);
    state.propose_check_is_contract_registered(&contract_name);
    state.propose_register_header_verifier(HEADER_VERIFIER_ADDRESS);
    state.propose_register_bls_keys(bls_keys, Some(error_status));
}

#[test]
fn register_bls_keys() {
    let mut state = LiquidStakingTestState::new();
    let contract_name = ManagedBuffer::from("delegation");
    let validator_1_bls_key = ManagedBuffer::from("bls_key_1");
    let validator_2_bls_key = ManagedBuffer::from("bls_key_2");
    let bls_keys =
        state.map_bls_key_vec_to_multi_value(vec![&validator_1_bls_key, &validator_2_bls_key]);

    state.propose_setup_contracts();
    state.propose_register_delegation_address(&contract_name, DELEGATION_ADDRESS, None);
    state.propose_register_header_verifier(HEADER_VERIFIER_ADDRESS);
    state.propose_register_bls_keys(bls_keys, None);

    state
        .world
        .query()
        .to(LIQUID_STAKING_ADDRESS)
        .whitebox(liquid_staking::contract_obj, |sc| {
            assert!(!sc.registered_bls_keys().is_empty());
        })
}

#[test]
fn slash_no_delegated_value() {
    let mut state = LiquidStakingTestState::new();
    let contract_name = ManagedBuffer::from("delegation");
    let validator_1_bls_key = ManagedBuffer::from("bls_key_1");
    let validator_2_bls_key = ManagedBuffer::from("bls_key_2");
    let bls_keys =
        state.map_bls_key_vec_to_multi_value(vec![&validator_1_bls_key, &validator_2_bls_key]);
    let value_to_slash = BigUint::from(10_000u64);
    let error_status = ErrorStatus {
        code: 4,
        error_message: "Caller has 0 delegated value",
    };

    state.propose_setup_contracts();
    state.propose_register_delegation_address(&contract_name, DELEGATION_ADDRESS, None);
    state.propose_register_header_verifier(HEADER_VERIFIER_ADDRESS);
    state.propose_register_bls_keys(bls_keys, None);
    state.whitebox_map_bls_to_address("bls_key_1", &VALIDATOR_ADDRESS);
    state.propose_slash_validator(&validator_1_bls_key, value_to_slash, Some(error_status));
}

#[test]
fn slash_zero_value() {
    let mut state = LiquidStakingTestState::new();
    let contract_name = ManagedBuffer::from("delegation");
    let validator_1_bls_key = ManagedBuffer::from("bls_key_1");
    let validator_2_bls_key = ManagedBuffer::from("bls_key_2");
    let payment = BigUint::from(100_000u64);
    let bls_keys =
        state.map_bls_key_vec_to_multi_value(vec![&validator_1_bls_key, &validator_2_bls_key]);
    let value_to_slash = BigUint::from(0u64);
    let error_status = ErrorStatus {
        code: 4,
        error_message: "You can't slash a value of 0 eGLD",
    };

    state.propose_setup_contracts();
    state.propose_register_delegation_address(&contract_name, DELEGATION_ADDRESS, None);
    state.propose_register_header_verifier(HEADER_VERIFIER_ADDRESS);
    state.propose_register_bls_keys(bls_keys, None);
    state.propose_stake(&VALIDATOR_ADDRESS, &contract_name, &payment);
    state.whitebox_map_bls_to_address("bls_key_1", &VALIDATOR_ADDRESS);
    state.propose_slash_validator(&validator_1_bls_key, value_to_slash, Some(error_status));
}

#[test]
fn slash_validator() {
    let mut state = LiquidStakingTestState::new();
    let contract_name = ManagedBuffer::from("delegation");
    let validator_1_bls_key = ManagedBuffer::from("bls_key_1");
    let validator_2_bls_key = ManagedBuffer::from("bls_key_2");
    let bls_keys =
        state.map_bls_key_vec_to_multi_value(vec![&validator_1_bls_key, &validator_2_bls_key]);
    let payment = BigUint::from(100_000u64);
    let value_to_slash = BigUint::from(10_000u64);

    state.propose_setup_contracts();
    state.propose_register_delegation_address(&contract_name, DELEGATION_ADDRESS, None);
    state.propose_register_header_verifier(HEADER_VERIFIER_ADDRESS);
    state.propose_register_bls_keys(bls_keys, None);
    state.propose_stake(&VALIDATOR_ADDRESS, &contract_name, &payment);
    state.whitebox_map_bls_to_address("bls_key_1", &VALIDATOR_ADDRESS);
    state.propose_slash_validator(&validator_1_bls_key, value_to_slash, None);

    state
        .world
        .query()
        .to(LIQUID_STAKING_ADDRESS)
        .whitebox(liquid_staking::contract_obj, |sc| {
            let expected_value = BigUint::from(90_000u64);
            let validator_delegated_value = sc
                .delegated_value(&VALIDATOR_ADDRESS.to_managed_address())
                .get();

            assert_eq!(validator_delegated_value, expected_value);
        })
}
