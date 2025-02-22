use multiversx_sc::types::{EsdtLocalRole, EsdtTokenPayment};
use multiversx_sc::{
    imports::{MultiValue3, OptionalValue},
    types::{BigUint, ManagedBuffer, ManagedVec, TokenIdentifier},
};
use multiversx_sc_scenario::api::StaticApi;
use multiversx_sc_scenario::ScenarioTxWhitebox;
use operation::{aliases::PaymentsVec, EsdtSafeConfig};
use proxies::fee_market_proxy::{FeeStruct, FeeType};
use setup::{
    SovEsdtSafeTestState, CONTRACT_ADDRESS, CONTRACT_CODE_PATH, FEE_MARKET_ADDRESS, FEE_TOKEN,
    HEADER_VERIFIER_ADDRESS, ONE_HUNDRED_MILLION, ONE_HUNDRED_THOUSAND, OWNER_ADDRESS,
    TEST_TOKEN_ONE, TEST_TOKEN_TWO, USER,
};
use sov_esdt_safe::SovEsdtSafe;

mod setup;

#[test]
fn deploy() {
    let mut state = SovEsdtSafeTestState::new();

    state.deploy_contract(HEADER_VERIFIER_ADDRESS, EsdtSafeConfig::default_config());
}

#[test]
fn deposit_no_fee() {
    let mut state = SovEsdtSafeTestState::new();

    state
        .world
        .account(CONTRACT_ADDRESS)
        .nonce(1)
        .code(CONTRACT_CODE_PATH)
        .owner(OWNER_ADDRESS)
        .esdt_roles(
            TokenIdentifier::from(TEST_TOKEN_ONE),
            vec![
                EsdtLocalRole::Burn.name().to_string(),
                EsdtLocalRole::NftBurn.name().to_string(),
            ],
        )
        .esdt_roles(
            TokenIdentifier::from(TEST_TOKEN_TWO),
            vec![
                EsdtLocalRole::Burn.name().to_string(),
                EsdtLocalRole::NftBurn.name().to_string(),
            ],
        )
        .esdt_roles(
            TokenIdentifier::from(FEE_TOKEN),
            vec![
                EsdtLocalRole::Burn.name().to_string(),
                EsdtLocalRole::NftBurn.name().to_string(),
            ],
        );

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(CONTRACT_ADDRESS)
        .whitebox(sov_esdt_safe::contract_obj, |sc| {
            let config = EsdtSafeConfig::new(
                ManagedVec::new(),
                ManagedVec::new(),
                50_000_000,
                ManagedVec::new(),
            );

            sc.init(HEADER_VERIFIER_ADDRESS.to_managed_address(), config);
        });

    state.deploy_fee_market(None);
    state.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_ONE),
        0,
        BigUint::from(ONE_HUNDRED_THOUSAND),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_TWO),
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

    state.world.check_account(OWNER_ADDRESS).esdt_balance(
        TokenIdentifier::from(TEST_TOKEN_ONE),
        &expected_amount_token_one,
    );

    let expected_amount_token_two =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_two.amount;

    state.world.check_account(OWNER_ADDRESS).esdt_balance(
        TokenIdentifier::from(TEST_TOKEN_TWO),
        &expected_amount_token_two,
    );
}

#[test]
fn deposit_with_fee() {
    let mut state = SovEsdtSafeTestState::new();

    state
        .world
        .account(CONTRACT_ADDRESS)
        .nonce(1)
        .code(CONTRACT_CODE_PATH)
        .owner(OWNER_ADDRESS)
        .esdt_roles(
            TokenIdentifier::from(TEST_TOKEN_ONE),
            vec![
                EsdtLocalRole::Burn.name().to_string(),
                EsdtLocalRole::NftBurn.name().to_string(),
            ],
        )
        .esdt_roles(
            TokenIdentifier::from(TEST_TOKEN_TWO),
            vec![
                EsdtLocalRole::Burn.name().to_string(),
                EsdtLocalRole::NftBurn.name().to_string(),
            ],
        )
        .esdt_roles(
            TokenIdentifier::from(FEE_TOKEN),
            vec![
                EsdtLocalRole::Burn.name().to_string(),
                EsdtLocalRole::NftBurn.name().to_string(),
            ],
        );

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(CONTRACT_ADDRESS)
        .whitebox(sov_esdt_safe::contract_obj, |sc| {
            let config = EsdtSafeConfig::new(
                ManagedVec::new(),
                ManagedVec::new(),
                50_000_000,
                ManagedVec::new(),
            );

            sc.init(HEADER_VERIFIER_ADDRESS.to_managed_address(), config);
        });

    let per_transfer = BigUint::from(100u64);
    let per_gas = BigUint::from(1u64);

    let fee = FeeStruct {
        base_token: TokenIdentifier::from(FEE_TOKEN),
        fee_type: FeeType::Fixed {
            token: TokenIdentifier::from(FEE_TOKEN),
            per_transfer: per_transfer.clone(),
            per_gas: per_gas.clone(),
        },
    };

    state.deploy_fee_market(Some(fee));
    state.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let fee_amount = BigUint::from(ONE_HUNDRED_THOUSAND);

    let fee_payment =
        EsdtTokenPayment::<StaticApi>::new(TokenIdentifier::from(FEE_TOKEN), 0, fee_amount.clone());

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_ONE),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_TWO),
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

    state.world.check_account(OWNER_ADDRESS).esdt_balance(
        TokenIdentifier::from(TEST_TOKEN_ONE),
        expected_amount_token_one,
    );

    let expected_amount_token_two =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_two.amount;

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
        .esdt_balance(TokenIdentifier::from(FEE_TOKEN), expected_amount_token_fee);
}
