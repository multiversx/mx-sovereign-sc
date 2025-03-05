use multiversx_sc::{
    imports::{MultiValue3, OptionalValue},
    types::{
        BigUint, EsdtTokenPayment, ManagedBuffer, ManagedVec, TestTokenIdentifier, TokenIdentifier,
    },
};

use multiversx_sc_scenario::api::StaticApi;
use proxies::fee_market_proxy::{FeeStruct, FeeType};
use sov_esdt_safe_setup::{
    SovEsdtSafeTestState, ESDT_SAFE_ADDRESS, FEE_MARKET_ADDRESS, FEE_TOKEN, ONE_HUNDRED_MILLION,
    ONE_HUNDRED_THOUSAND, OWNER_ADDRESS, TEST_TOKEN_ONE, TEST_TOKEN_TWO, USER,
};
use structs::{aliases::PaymentsVec, configs::EsdtSafeConfig};

mod sov_esdt_safe_setup;

#[test]
fn deploy() {
    let mut state = SovEsdtSafeTestState::new();

    state.deploy_contract(
        FEE_MARKET_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );
}

#[test]
fn deploy_no_config() {
    let mut state = SovEsdtSafeTestState::new();

    state.deploy_contract(FEE_MARKET_ADDRESS, OptionalValue::None);
    state
        .world
        .check_account(ESDT_SAFE_ADDRESS)
        .check_storage(
            "str:crossChainConfig",
            "0x00000000000000000000000011e1a30000000000", // default EsdtSafeConfig hex encoded
        )
        .check_storage(
            "str:feeMarketAddress",
            "0x000000000000000005006665652d6d61726b65745f5f5f5f5f5f5f5f5f5f5f5f", // FEE_MARKET_ADDRESS hex encoded, required for the check_storage to work
        );
}

#[test]
fn deploy_and_update_config() {
    let mut state = SovEsdtSafeTestState::new();

    state.deploy_contract(FEE_MARKET_ADDRESS, OptionalValue::None);

    state
        .world
        .check_account(ESDT_SAFE_ADDRESS)
        .check_storage(
            "str:crossChainConfig",
            "0x00000000000000000000000011e1a30000000000", // default EsdtSafeConfig hex encoded
        )
        .check_storage(
            "str:feeMarketAddress",
            "0x000000000000000005006665652d6d61726b65745f5f5f5f5f5f5f5f5f5f5f5f", // FEE_MARKET_ADDRESS hex encoded, required for the check_storage to work
        );

    let new_config = EsdtSafeConfig {
        token_whitelist: ManagedVec::from_single_item(TokenIdentifier::from(TEST_TOKEN_ONE)),
        token_blacklist: ManagedVec::from_single_item(TokenIdentifier::from(TEST_TOKEN_TWO)),
        max_tx_gas_limit: 30_000,
        banned_endpoints: ManagedVec::from_single_item(ManagedBuffer::from("endpoint")),
    };

    state.update_configuration(new_config, None);

    state
        .world
        .check_account(ESDT_SAFE_ADDRESS)
        .check_storage(
            "str:crossChainConfig",
            "0x000000010000000b544f4e452d313233343536000000010000000b5454574f2d31323334353600000000000075300000000100000008656e64706f696e74", // updated EsdtSafeConfig hex encoded
        )
        .check_storage(
            "str:feeMarketAddress",
            "0x000000000000000005006665652d6d61726b65745f5f5f5f5f5f5f5f5f5f5f5f", // FEE_MARKET_ADDRESS hex encoded, required for the check_storage to work
        );
}

#[test]
fn deposit_no_fee_no_transfer_data() {
    let mut state = SovEsdtSafeTestState::new();

    state.deploy_contract_with_roles();

    state.deploy_fee_market(None);
    state.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let test_token_one_identifier = TestTokenIdentifier::new(TEST_TOKEN_ONE);
    let test_token_two_identifier = TestTokenIdentifier::new(TEST_TOKEN_TWO);

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        test_token_one_identifier.into(),
        0,
        BigUint::from(ONE_HUNDRED_THOUSAND),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        test_token_two_identifier.into(),
        0,
        BigUint::from(ONE_HUNDRED_THOUSAND),
    );

    let payments_vec = PaymentsVec::from(vec![
        esdt_token_payment_one.clone(),
        esdt_token_payment_two.clone(),
    ]);

    let logs = state.deposit_with_logs(
        USER.to_managed_address(),
        OptionalValue::None,
        payments_vec.clone(),
    );

    for log in logs {
        assert!(!log.topics.is_empty());
    }

    state.check_sc_esdt_balance(
        vec![
            MultiValue3::from((test_token_one_identifier, 0u64, 0u64)),
            MultiValue3::from((test_token_two_identifier, 0u64, 0u64)),
        ],
        ESDT_SAFE_ADDRESS.to_managed_address(),
    );

    let expected_amount_token_one =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_one.amount;

    state
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(test_token_one_identifier, &expected_amount_token_one);

    let expected_amount_token_two =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_two.amount;

    state
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(test_token_two_identifier, &expected_amount_token_two);
}

#[test]
fn deposit_with_fee_no_transfer_data() {
    let mut state = SovEsdtSafeTestState::new();

    state.deploy_contract_with_roles();

    let per_transfer = BigUint::from(100u64);
    let per_gas = BigUint::from(1u64);
    let fee_token_identifier = TestTokenIdentifier::new(FEE_TOKEN);

    let fee = FeeStruct {
        base_token: fee_token_identifier.into(),
        fee_type: FeeType::Fixed {
            token: fee_token_identifier.into(),
            per_transfer: per_transfer.clone(),
            per_gas: per_gas.clone(),
        },
    };

    state.deploy_fee_market(Some(fee));
    state.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let test_token_one_identifier = TestTokenIdentifier::new(TEST_TOKEN_ONE);
    let test_token_two_identifier = TestTokenIdentifier::new(TEST_TOKEN_TWO);

    let fee_amount = BigUint::from(ONE_HUNDRED_THOUSAND);

    let fee_payment =
        EsdtTokenPayment::<StaticApi>::new(fee_token_identifier.into(), 0, fee_amount.clone());

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        test_token_one_identifier.into(),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        test_token_two_identifier.into(),
        0,
        BigUint::from(100u64),
    );

    let payments_vec = PaymentsVec::from(vec![
        fee_payment,
        esdt_token_payment_one.clone(),
        esdt_token_payment_two.clone(),
    ]);

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::None,
        Some(payments_vec.clone()),
        None,
    );

    let expected_amount_token_one =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_one.amount;

    state
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(test_token_one_identifier, expected_amount_token_one);

    let expected_amount_token_two =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_two.amount;

    state.check_sc_esdt_balance(
        vec![
            MultiValue3::from((test_token_one_identifier, 0u64, 0u64)),
            MultiValue3::from((test_token_two_identifier, 0u64, 0u64)),
        ],
        ESDT_SAFE_ADDRESS.to_managed_address(),
    );

    state
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(test_token_two_identifier, expected_amount_token_two);

    let expected_amount_token_fee =
        BigUint::from(ONE_HUNDRED_MILLION) - BigUint::from(payments_vec.len() - 1) * per_transfer;

    state
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(fee_token_identifier, expected_amount_token_fee);
}

#[test]
fn deposit_no_fee_with_transfer_data() {
    let mut state = SovEsdtSafeTestState::new();

    state.deploy_contract_with_roles();

    state.deploy_fee_market(None);
    state.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let test_token_one_identifier = TestTokenIdentifier::new(TEST_TOKEN_ONE);
    let test_token_two_identifier = TestTokenIdentifier::new(TEST_TOKEN_TWO);

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        test_token_one_identifier.into(),
        0,
        BigUint::from(ONE_HUNDRED_THOUSAND),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        test_token_two_identifier.into(),
        0,
        BigUint::from(ONE_HUNDRED_THOUSAND),
    );

    let payments_vec = PaymentsVec::from(vec![
        esdt_token_payment_one.clone(),
        esdt_token_payment_two.clone(),
    ]);

    let gas_limit = 1;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    let logs = state.deposit_with_logs(
        USER.to_managed_address(),
        OptionalValue::Some(transfer_data),
        payments_vec.clone(),
    );

    for log in logs {
        assert!(!log.topics.is_empty());
    }

    let expected_amount_token_one =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_one.amount;

    state.check_sc_esdt_balance(
        vec![
            MultiValue3::from((test_token_one_identifier, 0u64, 0u64)),
            MultiValue3::from((test_token_two_identifier, 0u64, 0u64)),
        ],
        ESDT_SAFE_ADDRESS.to_managed_address(),
    );

    state
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(test_token_one_identifier, &expected_amount_token_one);

    let expected_amount_token_two =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_two.amount;

    state.world.check_account(OWNER_ADDRESS).esdt_balance(
        TokenIdentifier::from(TEST_TOKEN_TWO),
        &expected_amount_token_two,
    );
}

#[test]
fn deposit_with_fee_with_transfer_data() {
    let mut state = SovEsdtSafeTestState::new();

    state.deploy_contract_with_roles();

    let per_transfer = BigUint::from(100u64);
    let per_gas = BigUint::from(1u64);
    let fee_token_identifier = TestTokenIdentifier::new(FEE_TOKEN);

    let fee = FeeStruct {
        base_token: fee_token_identifier.into(),
        fee_type: FeeType::Fixed {
            token: fee_token_identifier.into(),
            per_transfer: per_transfer.clone(),
            per_gas: per_gas.clone(),
        },
    };

    state.deploy_fee_market(Some(fee));
    state.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let test_token_one_identifier = TestTokenIdentifier::new(TEST_TOKEN_ONE);
    let test_token_two_identifier = TestTokenIdentifier::new(TEST_TOKEN_TWO);

    let fee_amount = BigUint::from(ONE_HUNDRED_THOUSAND);

    let fee_payment =
        EsdtTokenPayment::<StaticApi>::new(fee_token_identifier.into(), 0, fee_amount.clone());

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        test_token_one_identifier.into(),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        test_token_two_identifier.into(),
        0,
        BigUint::from(100u64),
    );

    let payments_vec = PaymentsVec::from(vec![
        fee_payment,
        esdt_token_payment_one.clone(),
        esdt_token_payment_two.clone(),
    ]);

    let gas_limit = 2;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::Some(transfer_data),
        Some(payments_vec.clone()),
        None,
    );

    let expected_amount_token_one =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_one.amount;

    state
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(test_token_one_identifier, expected_amount_token_one);

    let expected_amount_token_two =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_two.amount;

    state.check_sc_esdt_balance(
        vec![
            MultiValue3::from((test_token_one_identifier, 0u64, 0u64)),
            MultiValue3::from((test_token_two_identifier, 0u64, 0u64)),
        ],
        ESDT_SAFE_ADDRESS.to_managed_address(),
    );

    state.world.check_account(OWNER_ADDRESS).esdt_balance(
        TokenIdentifier::from(TEST_TOKEN_TWO),
        expected_amount_token_two,
    );

    let expected_amount_token_fee = BigUint::from(ONE_HUNDRED_MILLION)
        - BigUint::from(payments_vec.len() - 1) * per_transfer
        - BigUint::from(gas_limit) * per_gas;

    state
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(fee_token_identifier, expected_amount_token_fee);
}
