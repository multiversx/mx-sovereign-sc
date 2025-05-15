use common_test_setup::constants::{
    ESDT_SAFE_ADDRESS, FEE_MARKET_ADDRESS, FEE_TOKEN, FIRST_TEST_TOKEN, ONE_HUNDRED_MILLION,
    ONE_HUNDRED_THOUSAND, OWNER_ADDRESS, SECOND_TEST_TOKEN, USER_ADDRESS,
};
use error_messages::NOTHING_TO_TRANSFER;
use multiversx_sc::{
    imports::{MultiValue3, OptionalValue},
    types::{BigUint, EsdtTokenPayment, ManagedBuffer, ManagedVec, MultiValueEncoded},
};
use multiversx_sc_scenario::api::StaticApi;
use sov_esdt_safe_blackbox_setup::SovEsdtSafeTestState;
use structs::{
    aliases::PaymentsVec,
    configs::EsdtSafeConfig,
    fee::{FeeStruct, FeeType},
};

mod sov_esdt_safe_blackbox_setup;

#[test]
fn test_deploy() {
    let mut state = SovEsdtSafeTestState::new();

    state.common_setup.deploy_sov_esdt_safe(
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
fn test_deposit_no_fee_no_transfer_data() {
    let mut state = SovEsdtSafeTestState::new();

    state.deploy_contract_with_roles();

    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        FIRST_TEST_TOKEN.into(),
        0,
        BigUint::from(ONE_HUNDRED_THOUSAND),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        SECOND_TEST_TOKEN.into(),
        0,
        BigUint::from(ONE_HUNDRED_THOUSAND),
    );

    let payments_vec = PaymentsVec::from(vec![
        esdt_token_payment_one.clone(),
        esdt_token_payment_two.clone(),
    ]);

    state.deposit_with_logs(
        USER_ADDRESS.to_managed_address(),
        OptionalValue::None,
        payments_vec.clone(),
        None,
        Some("deposit"),
    );

    let expected_tokens = vec![
        MultiValue3::from((FIRST_TEST_TOKEN, 0u64, BigUint::from(0u64))),
        MultiValue3::from((SECOND_TEST_TOKEN, 0u64, BigUint::from(0u64))),
    ];

    state
        .common_setup
        .check_account_multiple_esdts(ESDT_SAFE_ADDRESS.to_address(), expected_tokens);

    let expected_amount_token_one =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_one.amount;

    let expected_amount_token_two =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_two.amount;

    let expected_balances = vec![
        MultiValue3::from((FIRST_TEST_TOKEN, 0u64, expected_amount_token_one)),
        MultiValue3::from((SECOND_TEST_TOKEN, 0u64, expected_amount_token_two)),
    ];

    state
        .common_setup
        .check_account_multiple_esdts(OWNER_ADDRESS.to_address(), expected_balances);
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
fn test_deposit_with_fee_no_transfer_data() {
    let mut state = SovEsdtSafeTestState::new();

    state.deploy_contract_with_roles();

    let per_transfer = BigUint::from(100u64);
    let per_gas = BigUint::from(1u64);
    let fee_token_identifier = FEE_TOKEN;

    let fee = FeeStruct {
        base_token: fee_token_identifier.into(),
        fee_type: FeeType::Fixed {
            token: fee_token_identifier.into(),
            per_transfer: per_transfer.clone(),
            per_gas: per_gas.clone(),
        },
    };

    state
        .common_setup
        .deploy_fee_market(Some(fee), ESDT_SAFE_ADDRESS);
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let fee_amount = BigUint::from(ONE_HUNDRED_THOUSAND);

    let fee_payment =
        EsdtTokenPayment::<StaticApi>::new(fee_token_identifier.into(), 0, fee_amount.clone());

    let esdt_token_payment_one =
        EsdtTokenPayment::<StaticApi>::new(FIRST_TEST_TOKEN.into(), 0, BigUint::from(100u64));

    let esdt_token_payment_two =
        EsdtTokenPayment::<StaticApi>::new(SECOND_TEST_TOKEN.into(), 0, BigUint::from(100u64));

    let payments_vec = PaymentsVec::from(vec![
        fee_payment,
        esdt_token_payment_one.clone(),
        esdt_token_payment_two.clone(),
    ]);

    state.deposit(
        USER_ADDRESS.to_managed_address(),
        OptionalValue::None,
        payments_vec.clone(),
        None,
        Some("deposit"),
    );

    let expected_amount_token_one =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_one.amount;

    state.common_setup.check_account_single_esdt(
        OWNER_ADDRESS.to_address(),
        FIRST_TEST_TOKEN,
        0,
        expected_amount_token_one,
    );

    let expected_amount_token_two =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_two.amount;

    let expected_tokens = vec![
        MultiValue3::from((FIRST_TEST_TOKEN, 0u64, BigUint::from(0u64))),
        MultiValue3::from((SECOND_TEST_TOKEN, 0u64, BigUint::from(0u64))),
    ];

    state
        .common_setup
        .check_account_multiple_esdts(ESDT_SAFE_ADDRESS.to_address(), expected_tokens);

    state.common_setup.check_account_single_esdt(
        OWNER_ADDRESS.to_address(),
        SECOND_TEST_TOKEN,
        0u64,
        expected_amount_token_two,
    );

    let expected_amount_token_fee =
        BigUint::from(ONE_HUNDRED_MILLION) - BigUint::from(payments_vec.len() - 1) * per_transfer;

    state.common_setup.check_account_single_esdt(
        OWNER_ADDRESS.to_address(),
        fee_token_identifier,
        0u64,
        expected_amount_token_fee,
    );
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
fn test_deposit_no_fee_with_transfer_data() {
    let mut state = SovEsdtSafeTestState::new();

    state.deploy_contract_with_roles();

    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        FIRST_TEST_TOKEN.into(),
        0,
        BigUint::from(ONE_HUNDRED_THOUSAND),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        SECOND_TEST_TOKEN.into(),
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
        USER_ADDRESS.to_managed_address(),
        OptionalValue::Some(transfer_data),
        payments_vec.clone(),
        None,
        Some("deposit"),
    );

    let expected_amount_token_one =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_one.amount;

    let expected_tokens = vec![
        MultiValue3::from((FIRST_TEST_TOKEN, 0u64, BigUint::from(0u64))),
        MultiValue3::from((SECOND_TEST_TOKEN, 0u64, BigUint::from(0u64))),
    ];

    state
        .common_setup
        .check_account_multiple_esdts(ESDT_SAFE_ADDRESS.to_address(), expected_tokens);

    state.common_setup.check_account_single_esdt(
        OWNER_ADDRESS.to_address(),
        FIRST_TEST_TOKEN,
        0u64,
        expected_amount_token_one,
    );

    let expected_amount_token_two =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_two.amount;

    state.common_setup.check_account_single_esdt(
        OWNER_ADDRESS.to_address(),
        SECOND_TEST_TOKEN,
        0u64,
        expected_amount_token_two,
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
fn test_deposit_with_fee_with_transfer_data() {
    let mut state = SovEsdtSafeTestState::new();

    state.deploy_contract_with_roles();

    let per_transfer = BigUint::from(100u64);
    let per_gas = BigUint::from(1u64);
    let fee_token_identifier = FEE_TOKEN;

    let fee = FeeStruct {
        base_token: fee_token_identifier.into(),
        fee_type: FeeType::Fixed {
            token: fee_token_identifier.into(),
            per_transfer: per_transfer.clone(),
            per_gas: per_gas.clone(),
        },
    };

    state
        .common_setup
        .deploy_fee_market(Some(fee), ESDT_SAFE_ADDRESS);
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let fee_amount = BigUint::from(ONE_HUNDRED_THOUSAND);

    let fee_payment =
        EsdtTokenPayment::<StaticApi>::new(fee_token_identifier.into(), 0, fee_amount.clone());

    let esdt_token_payment_one =
        EsdtTokenPayment::<StaticApi>::new(FIRST_TEST_TOKEN.into(), 0, BigUint::from(100u64));

    let esdt_token_payment_two =
        EsdtTokenPayment::<StaticApi>::new(SECOND_TEST_TOKEN.into(), 0, BigUint::from(100u64));

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
        USER_ADDRESS.to_managed_address(),
        OptionalValue::Some(transfer_data),
        payments_vec.clone(),
        None,
        Some("deposit"),
    );

    let expected_amount_token_one =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_one.amount;

    state.common_setup.check_account_single_esdt(
        OWNER_ADDRESS.to_address(),
        FIRST_TEST_TOKEN,
        0u64,
        expected_amount_token_one,
    );

    let expected_amount_token_two =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_two.amount;

    let expected_tokens = vec![
        MultiValue3::from((FIRST_TEST_TOKEN, 0u64, BigUint::from(0u64))),
        MultiValue3::from((SECOND_TEST_TOKEN, 0u64, BigUint::from(0u64))),
    ];

    state
        .common_setup
        .check_account_multiple_esdts(ESDT_SAFE_ADDRESS.to_address(), expected_tokens);

    state.common_setup.check_account_single_esdt(
        OWNER_ADDRESS.to_address(),
        SECOND_TEST_TOKEN,
        0u64,
        expected_amount_token_two,
    );

    let expected_amount_token_fee = BigUint::from(ONE_HUNDRED_MILLION)
        - BigUint::from(payments_vec.len() - 1) * per_transfer
        - BigUint::from(gas_limit) * per_gas;

    state.common_setup.check_account_single_esdt(
        OWNER_ADDRESS.to_address(),
        fee_token_identifier,
        0u64,
        expected_amount_token_fee,
    );
}

/// Test the deposit function with no transfer data and no payments
#[test]
fn test_deposit_no_transfer_data_no_payments() {
    let mut state = SovEsdtSafeTestState::new();

    state.deploy_contract_with_roles();
    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    state.deposit(
        USER_ADDRESS.to_managed_address(),
        OptionalValue::None,
        PaymentsVec::new(),
        Some(NOTHING_TO_TRANSFER),
        None,
    );
}

/// Test the deposit function with only cross-chain sc call
/// Steps:
/// 1. Deploy the Sov-ESDT-Safe smart contract with roles.
/// 2. Deploy the Fee-Market smart contract.
/// 3. Deploy the Testing smart contract.
/// 5. Set the Fee-Market address.
/// 6. Create the cross-chain sc call TransferData
/// 7. Check for the `scCall` log
#[test]
fn test_deposit_sc_call_only() {
    let mut state = SovEsdtSafeTestState::new();

    state.deploy_contract_with_roles();
    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);
    state.common_setup.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let gas_limit = 2;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        MultiValueEncoded::<StaticApi, ManagedBuffer<StaticApi>>::from(ManagedVec::from(vec![
            ManagedBuffer::from("1"),
        ]));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    state.deposit(
        USER_ADDRESS.to_managed_address(),
        OptionalValue::Some(transfer_data.clone()),
        PaymentsVec::new(),
        None,
        Some("scCall"),
    );
}
