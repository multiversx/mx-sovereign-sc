use common_tests_setup::{
    ESDT_SAFE_ADDRESS, FEE_MARKET_ADDRESS, FEE_TOKEN, FIRST_TEST_TOKEN, ONE_HUNDRED_MILLION,
    ONE_HUNDRED_THOUSAND, OWNER_ADDRESS, SECOND_TEST_TOKEN, USER,
};
use multiversx_sc::{
    imports::{MultiValue3, OptionalValue},
    types::{
        BigUint, EsdtTokenPayment, ManagedBuffer, ManagedVec, MultiValueEncoded,
        TestTokenIdentifier, TokenIdentifier,
    },
};
use multiversx_sc_scenario::api::StaticApi;
use proxies::fee_market_proxy::{FeeStruct, FeeType};
use sov_esdt_safe_setup::SovEsdtSafeTestState;
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

/// Test the deposit function without fee and without transfer data.
/// Steps:
/// 1. Deploy the Sov-ESDT-Safe smart contract with roles.
/// 2. Deploy the Fee-Market smart contract.
/// 3. Deploy the Testing smart contract.
/// 4. Set the Fee-Market address.
/// 5. Create two ESDT token payments.
/// 6. Create a payments vector with the two ESDT token payments.
/// 7. Call the deposit function with the payments vector.
/// 8. Check the logs for the deposit function.
/// 9. Check the ESDT balance of the addresses
#[test]
fn deposit_no_fee_no_transfer_data() {
    let mut state = SovEsdtSafeTestState::new();

    state.deploy_contract_with_roles();

    state.common_setup.deploy_fee_market(None);
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let test_token_one_identifier = TestTokenIdentifier::new(FIRST_TEST_TOKEN);
    let test_token_two_identifier = TestTokenIdentifier::new(SECOND_TEST_TOKEN);

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

    state.deposit_with_logs(
        USER.to_managed_address(),
        OptionalValue::None,
        payments_vec.clone(),
        None,
        Some("deposit"),
    );

    state.common_setup.check_sc_esdt_balance(
        vec![
            MultiValue3::from((test_token_one_identifier, 0u64, 0u64)),
            MultiValue3::from((test_token_two_identifier, 0u64, 0u64)),
        ],
        ESDT_SAFE_ADDRESS.to_managed_address(),
        sov_esdt_safe::contract_obj,
    );

    let expected_amount_token_one =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_one.amount;

    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(test_token_one_identifier, &expected_amount_token_one);

    let expected_amount_token_two =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_two.amount;

    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(test_token_two_identifier, &expected_amount_token_two);
}

/// Test the deposit function with fee and without transfer data.
/// Steps:
/// 1. Deploy the Sov-ESDT-Safe smart contract with roles.
/// 2. Deploy the Fee-Market smart contract.
/// 3. Deploy the Testing smart contract.
/// 4. Set the Fee-Market address.
/// 5. Create a fee payment.
/// 6. Create two ESDT token payments.
/// 7. Create a payments vector with the fee payment and the two ESDT token payments.
/// 8. Call the deposit function with the payments vector.
/// 9. Check the ESDT balances of the addresses
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

    state.common_setup.deploy_fee_market(Some(fee));
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let test_token_one_identifier = TestTokenIdentifier::new(FIRST_TEST_TOKEN);
    let test_token_two_identifier = TestTokenIdentifier::new(SECOND_TEST_TOKEN);

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
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(test_token_one_identifier, expected_amount_token_one);

    let expected_amount_token_two =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_two.amount;

    state.common_setup.check_sc_esdt_balance(
        vec![
            MultiValue3::from((test_token_one_identifier, 0u64, 0u64)),
            MultiValue3::from((test_token_two_identifier, 0u64, 0u64)),
        ],
        ESDT_SAFE_ADDRESS.to_managed_address(),
        sov_esdt_safe::contract_obj,
    );

    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(test_token_two_identifier, expected_amount_token_two);

    let expected_amount_token_fee =
        BigUint::from(ONE_HUNDRED_MILLION) - BigUint::from(payments_vec.len() - 1) * per_transfer;

    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(fee_token_identifier, expected_amount_token_fee);
}

/// Test the deposit function without fee and with transfer data.
/// Steps:
/// 1. Deploy the Sov-ESDT-Safe smart contract with roles.
/// 2. Deploy the Fee-Market smart contract.
/// 3. Deploy the Testing smart contract.
/// 4. Set the Fee-Market address.
/// 5. Create two ESDT token payments.
/// 6. Create a payments vector with the two ESDT token payments.
/// 7. Call the deposit function with the payments vector.
/// 8. Check the logs for the deposit function.
/// 9. Check the ESDT balance of the addresses
#[test]
fn deposit_no_fee_with_transfer_data() {
    let mut state = SovEsdtSafeTestState::new();

    state.deploy_contract_with_roles();

    state.common_setup.deploy_fee_market(None);
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let test_token_one_identifier = TestTokenIdentifier::new(FIRST_TEST_TOKEN);
    let test_token_two_identifier = TestTokenIdentifier::new(SECOND_TEST_TOKEN);

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
        MultiValueEncoded::<StaticApi, ManagedBuffer<StaticApi>>::from(ManagedVec::from(vec![
            ManagedBuffer::from("1"),
        ]));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    state.deposit_with_logs(
        USER.to_managed_address(),
        OptionalValue::Some(transfer_data),
        payments_vec.clone(),
        None,
        Some("deposit"),
    );

    let expected_amount_token_one =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_one.amount;

    state.common_setup.check_sc_esdt_balance(
        vec![
            MultiValue3::from((test_token_one_identifier, 0u64, 0u64)),
            MultiValue3::from((test_token_two_identifier, 0u64, 0u64)),
        ],
        ESDT_SAFE_ADDRESS.to_managed_address(),
        sov_esdt_safe::contract_obj,
    );

    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(test_token_one_identifier, &expected_amount_token_one);

    let expected_amount_token_two =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_two.amount;

    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(
            TokenIdentifier::from(SECOND_TEST_TOKEN),
            &expected_amount_token_two,
        );
}

/// Test the deposit function with fee and with transfer data.
/// Steps:
/// 1. Deploy the Sov-ESDT-Safe smart contract with roles.
/// 2. Deploy the Fee-Market smart contract.
/// 3. Deploy the Testing smart contract.
/// 4. Set the Fee-Market address.
/// 5. Create a fee payment.
/// 6. Create two ESDT token payments.
/// 7. Create a payments vector with the fee payment and the two ESDT token payments.
/// 8. Call the deposit function with the payments vector.
/// 9. Check the ESDT balances of the addresses
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

    state.common_setup.deploy_fee_market(Some(fee));
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let test_token_one_identifier = TestTokenIdentifier::new(FIRST_TEST_TOKEN);
    let test_token_two_identifier = TestTokenIdentifier::new(SECOND_TEST_TOKEN);

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
        MultiValueEncoded::<StaticApi, ManagedBuffer<StaticApi>>::from(ManagedVec::from(vec![
            ManagedBuffer::from("1"),
        ]));

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
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(test_token_one_identifier, expected_amount_token_one);

    let expected_amount_token_two =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_two.amount;

    state.common_setup.check_sc_esdt_balance(
        vec![
            MultiValue3::from((test_token_one_identifier, 0u64, 0u64)),
            MultiValue3::from((test_token_two_identifier, 0u64, 0u64)),
        ],
        ESDT_SAFE_ADDRESS.to_managed_address(),
        sov_esdt_safe::contract_obj,
    );

    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(
            TokenIdentifier::from(SECOND_TEST_TOKEN),
            expected_amount_token_two,
        );

    let expected_amount_token_fee = BigUint::from(ONE_HUNDRED_MILLION)
        - BigUint::from(payments_vec.len() - 1) * per_transfer
        - BigUint::from(gas_limit) * per_gas;

    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(fee_token_identifier, expected_amount_token_fee);
}
