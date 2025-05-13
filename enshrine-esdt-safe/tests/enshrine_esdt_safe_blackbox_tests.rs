use common_test_setup::constants::{
    CROWD_TOKEN_ID, ENSHRINE_BALANCE, FUNGIBLE_TOKEN_ID, ISSUE_COST, NFT_TOKEN_ID, OWNER_ADDRESS,
    PREFIX_NFT_TOKEN_ID, USER_ADDRESS, WEGLD_IDENTIFIER,
};
use enshrine_esdt_safe_blackbox_setup::EnshrineTestState;
use error_messages::{
    ACTION_IS_NOT_ALLOWED, BANNED_ENDPOINT_NAME, INSUFFICIENT_FUNDS, NOTHING_TO_TRANSFER,
    NOT_ENOUGH_WEGLD_AMOUNT, ONLY_WEGLD_IS_ACCEPTED_AS_REGISTER_FEE, PAYMENT_DOES_NOT_COVER_FEE,
    TOO_MANY_TOKENS,
};
use multiversx_sc::imports::{MultiValue3, OptionalValue};
use multiversx_sc::types::{
    BigUint, EsdtTokenPayment, ManagedBuffer, ManagedVec, MultiValueEncoded,
};
use structs::aliases::PaymentsVec;
use structs::configs::EsdtSafeConfig;

mod enshrine_esdt_safe_blackbox_setup;

#[test]
fn test_deploy() {
    let mut state = EnshrineTestState::new();

    state.setup_contracts(false, None, None);
}

#[test]
fn test_sovereign_prefix_no_prefix() {
    let mut state = EnshrineTestState::new();
    let token_vec = Vec::from([NFT_TOKEN_ID, CROWD_TOKEN_ID]);

    state.setup_contracts(false, None, None);
    state.register_operation(&token_vec);
    state.register_esdt_in_header_verifier();
    state.whitelist_enshrine_esdt();
    state.execute_operation(Some(ACTION_IS_NOT_ALLOWED), &token_vec);
}

#[test]
fn test_sovereign_prefix_has_prefix() {
    let mut state = EnshrineTestState::new();
    let token_vec = Vec::from([PREFIX_NFT_TOKEN_ID, CROWD_TOKEN_ID]);

    state.setup_contracts(false, None, None);
    state.register_operation(&token_vec);
    state.register_esdt_in_header_verifier();
    state.whitelist_enshrine_esdt();
    state.execute_operation(None, &token_vec);
}

#[test]
fn test_register_tokens_insufficient_funds() {
    let mut state = EnshrineTestState::new();
    let token_vec = Vec::from([PREFIX_NFT_TOKEN_ID, CROWD_TOKEN_ID]);
    let payment_amount = BigUint::from(ISSUE_COST * token_vec.len() as u64);
    let payment = EsdtTokenPayment::new(WEGLD_IDENTIFIER.into(), 0, payment_amount);

    state.setup_contracts(false, None, None);
    state.register_tokens(&USER_ADDRESS, payment, token_vec, Some(INSUFFICIENT_FUNDS));
}

#[test]
fn test_register_tokens_wrong_token_as_fee() {
    let mut state = EnshrineTestState::new();
    let token_vec = Vec::from([PREFIX_NFT_TOKEN_ID, CROWD_TOKEN_ID]);
    let payment_amount = BigUint::from(ISSUE_COST * token_vec.len() as u64);
    let payment = EsdtTokenPayment::new(CROWD_TOKEN_ID.into(), 0, payment_amount);

    state.setup_contracts(false, None, None);
    state.register_tokens(
        &OWNER_ADDRESS,
        payment,
        token_vec,
        Some(ONLY_WEGLD_IS_ACCEPTED_AS_REGISTER_FEE),
    );
}

#[test]
fn test_register_tokens() {
    let mut state = EnshrineTestState::new();
    let token_vec = Vec::from([PREFIX_NFT_TOKEN_ID, CROWD_TOKEN_ID]);
    let payment_amount = BigUint::from(ISSUE_COST * token_vec.len() as u64);
    let payment = EsdtTokenPayment::new(WEGLD_IDENTIFIER.into(), 0, payment_amount);

    state.setup_contracts(false, None, None);
    state.register_tokens(&OWNER_ADDRESS, payment, token_vec, None);
    state
        .common_setup
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(WEGLD_IDENTIFIER, BigUint::zero());
}

#[test]
fn test_register_tokens_insufficient_wegld() {
    let mut state = EnshrineTestState::new();
    let token_vec = Vec::from([
        NFT_TOKEN_ID,
        PREFIX_NFT_TOKEN_ID,
        FUNGIBLE_TOKEN_ID,
        CROWD_TOKEN_ID,
    ]);
    let payment_amount = BigUint::from(ISSUE_COST + token_vec.len() as u64);
    let payment = EsdtTokenPayment::new(WEGLD_IDENTIFIER.into(), 0, payment_amount);

    state.setup_contracts(false, None, None);
    state.register_tokens(
        &OWNER_ADDRESS,
        payment,
        token_vec,
        Some(NOT_ENOUGH_WEGLD_AMOUNT),
    );
}

#[test]
fn test_deposit_no_fee() {
    let mut state = EnshrineTestState::new();
    let amount = BigUint::from(10000u64);
    let wegld_payment = EsdtTokenPayment::new(WEGLD_IDENTIFIER.into(), 0, amount.clone());
    let mut payments = PaymentsVec::new();

    payments.push(wegld_payment);

    state.setup_contracts(false, None, None);
    state.set_fee(None, None);
    state.deposit(
        OWNER_ADDRESS,
        USER_ADDRESS,
        payments,
        OptionalValue::None,
        None,
    );
}

#[test]
fn test_deposit_token_nothing_to_transfer_fee_disabled() {
    let mut state = EnshrineTestState::new();
    let payments = PaymentsVec::new();

    state.setup_contracts(false, None, None);
    state.deposit(
        OWNER_ADDRESS,
        USER_ADDRESS,
        payments,
        OptionalValue::None,
        Some(NOTHING_TO_TRANSFER),
    );
}

#[test]
fn test_deposit_max_transfers_exceeded() {
    let mut state = EnshrineTestState::new();
    let amount = BigUint::from(10000u64);
    let wegld_payment = EsdtTokenPayment::new(WEGLD_IDENTIFIER.into(), 0, amount.clone());
    let mut payments = PaymentsVec::new();
    payments.extend(vec![wegld_payment; 11]);

    state.setup_contracts(false, None, None);
    state.deposit(
        OWNER_ADDRESS,
        USER_ADDRESS,
        payments,
        OptionalValue::None,
        Some(TOO_MANY_TOKENS),
    );
}

#[test]
fn test_deposit_no_transfer_data() {
    let mut state = EnshrineTestState::new();
    let amount = BigUint::from(10000u64);
    let wegld_payment = EsdtTokenPayment::new(WEGLD_IDENTIFIER.into(), 0, amount.clone());
    let fungible_payment = EsdtTokenPayment::new(FUNGIBLE_TOKEN_ID.into(), 0, amount.clone());
    let crowd_payment = EsdtTokenPayment::new(CROWD_TOKEN_ID.into(), 0, amount.clone());
    let mut payments = PaymentsVec::new();
    let mut tokens_whitelist = MultiValueEncoded::new();
    tokens_whitelist.push(WEGLD_IDENTIFIER.into());
    tokens_whitelist.push(CROWD_TOKEN_ID.into());

    payments.push(wegld_payment);
    payments.push(fungible_payment);
    payments.push(crowd_payment);

    let fee_amount_per_transfer = BigUint::from(100u32);
    let fee_amount_per_gas = BigUint::from(100u32);

    let fee_struct = state.setup_fee_struct(
        WEGLD_IDENTIFIER,
        &fee_amount_per_transfer,
        &fee_amount_per_gas,
    );

    state.setup_contracts(false, Some(&fee_struct), None);
    state.add_token_to_whitelist(tokens_whitelist);
    state.set_fee(Some(&fee_struct), None);
    state.deposit(
        OWNER_ADDRESS,
        USER_ADDRESS,
        payments,
        OptionalValue::None,
        None,
    );

    let expected_wegld_amount = BigUint::from(ENSHRINE_BALANCE) - fee_amount_per_transfer;
    let expected_crowd_amount = BigUint::from(ENSHRINE_BALANCE) - &amount;

    let expected_balances = vec![
        MultiValue3::from((WEGLD_IDENTIFIER, 0u64, expected_wegld_amount.clone())),
        MultiValue3::from((FUNGIBLE_TOKEN_ID, 0u64, BigUint::from(ENSHRINE_BALANCE))),
        MultiValue3::from((CROWD_TOKEN_ID, 0u64, expected_crowd_amount.clone())),
    ];

    state
        .common_setup
        .check_account_multiple_esdts(OWNER_ADDRESS.to_address(), expected_balances);
}

#[test]
fn test_deposit_with_transfer_data_gas_limit_too_high() {
    let mut state = EnshrineTestState::new();
    let amount = BigUint::from(10000u64);
    let wegld_payment = EsdtTokenPayment::new(WEGLD_IDENTIFIER.into(), 0, amount.clone());
    let crowd_payment = EsdtTokenPayment::new(CROWD_TOKEN_ID.into(), 0, amount);
    let mut payments = PaymentsVec::new();
    let gas_limit = 1000000000000000000;
    let function = ManagedBuffer::from("some_function");
    let arg = ManagedBuffer::from("arg");
    let mut args = ManagedVec::new();
    args.push(arg);

    let transfer_data = state.setup_transfer_data(gas_limit, function, args);

    payments.push(wegld_payment);
    payments.push(crowd_payment);

    state.setup_contracts(false, None, None);
    state.deposit(
        OWNER_ADDRESS,
        USER_ADDRESS,
        payments,
        transfer_data,
        Some("Gas limit too high"),
    );
}

#[test]
fn test_deposit_with_transfer_data_banned_endpoint() {
    let mut state = EnshrineTestState::new();
    let amount = BigUint::from(10000u64);
    let wegld_payment = EsdtTokenPayment::new(WEGLD_IDENTIFIER.into(), 0, amount.clone());
    let crowd_payment = EsdtTokenPayment::new(CROWD_TOKEN_ID.into(), 0, amount);
    let mut payments = PaymentsVec::new();
    let gas_limit = 1000000000;
    let banned_endpoint = ManagedBuffer::from("some_function");
    let arg = ManagedBuffer::from("arg");
    let mut args = ManagedVec::new();
    args.push(arg);

    let transfer_data = state.setup_transfer_data(gas_limit, banned_endpoint.clone(), args);

    payments.push(wegld_payment);
    payments.push(crowd_payment);

    state.setup_contracts(
        false,
        None,
        Some(EsdtSafeConfig::new(
            ManagedVec::new(),
            ManagedVec::new(),
            300_000_000_000,
            ManagedVec::from(vec![banned_endpoint]),
            ManagedVec::new(),
        )),
    );

    state.deposit(
        OWNER_ADDRESS,
        USER_ADDRESS,
        payments,
        transfer_data,
        Some(BANNED_ENDPOINT_NAME),
    );
}

#[test]
fn test_deposit_with_transfer_data_enough_for_fee() {
    let mut state = EnshrineTestState::new();
    let amount = BigUint::from(1000000000000000u128);
    let wegld_payment = EsdtTokenPayment::new(WEGLD_IDENTIFIER.into(), 0, amount.clone());
    let fungible_payment = EsdtTokenPayment::new(FUNGIBLE_TOKEN_ID.into(), 0, amount.clone());
    let crowd_payment = EsdtTokenPayment::new(CROWD_TOKEN_ID.into(), 0, amount.clone());
    let mut payments = PaymentsVec::new();
    let gas_limit = 10000000;
    let function = ManagedBuffer::from("some_function");
    let arg = ManagedBuffer::from("arg");
    let mut args = ManagedVec::new();
    args.push(arg);

    let transfer_data = state.setup_transfer_data(gas_limit, function, args);

    let expected_crowd_amount = BigUint::from(ENSHRINE_BALANCE) - &wegld_payment.amount;
    let expected_fungible_amount = BigUint::from(ENSHRINE_BALANCE) - &fungible_payment.amount;

    payments.push(wegld_payment);
    payments.push(fungible_payment);
    payments.push(crowd_payment);

    let fee_amount_per_transfer = BigUint::from(100u32);
    let fee_amount_per_gas = BigUint::from(100u32);

    let fee_struct = state.setup_fee_struct(
        WEGLD_IDENTIFIER,
        &fee_amount_per_transfer,
        &fee_amount_per_gas,
    );

    state.setup_contracts(false, Some(&fee_struct), None);
    // state.set_max_user_tx_gas_limit(gas_limit);
    state.set_fee(Some(&fee_struct), None);
    state.deposit(OWNER_ADDRESS, USER_ADDRESS, payments, transfer_data, None);

    let fee = fee_amount_per_transfer * BigUint::from(2u32)
        + BigUint::from(gas_limit) * fee_amount_per_gas;
    let expected_wegld_amount = BigUint::from(ENSHRINE_BALANCE) - fee;

    let expected_balances = vec![
        MultiValue3::from((WEGLD_IDENTIFIER, 0u64, expected_wegld_amount)),
        MultiValue3::from((FUNGIBLE_TOKEN_ID, 0u64, expected_fungible_amount)),
        MultiValue3::from((CROWD_TOKEN_ID, 0u64, expected_crowd_amount)),
    ];

    state
        .common_setup
        .check_account_multiple_esdts(OWNER_ADDRESS.to_address(), expected_balances);
}

#[test]
fn test_deposit_with_transfer_data_not_enough_for_fee() {
    let mut state = EnshrineTestState::new();
    let amount = BigUint::from(100000000000000000u128);
    let wegld_payment = EsdtTokenPayment::new(WEGLD_IDENTIFIER.into(), 0, BigUint::zero());
    let fungible_payment = EsdtTokenPayment::new(FUNGIBLE_TOKEN_ID.into(), 0, amount.clone());
    let crowd_payment = EsdtTokenPayment::new(CROWD_TOKEN_ID.into(), 0, amount.clone());
    let mut payments = PaymentsVec::new();
    let gas_limit = 10000000;
    let function = ManagedBuffer::from("some_function");
    let arg = ManagedBuffer::from("arg");
    let mut args = ManagedVec::new();
    args.push(arg);

    let transfer_data = state.setup_transfer_data(gas_limit, function, args);

    payments.push(wegld_payment);
    payments.push(fungible_payment);
    payments.push(crowd_payment);

    let fee_amount_per_transfer = BigUint::from(100u32);
    let fee_amount_per_gas = BigUint::from(100u32);

    let fee_struct = state.setup_fee_struct(
        WEGLD_IDENTIFIER,
        &fee_amount_per_transfer,
        &fee_amount_per_gas,
    );

    state.setup_contracts(false, Some(&fee_struct), None);
    // state.set_max_user_tx_gas_limit(gas_limit);
    state.set_fee(Some(&fee_struct), None);
    state.deposit(
        OWNER_ADDRESS,
        USER_ADDRESS,
        payments,
        transfer_data,
        Some(PAYMENT_DOES_NOT_COVER_FEE),
    );
}

#[test]
fn test_deposit_refund_non_whitelisted_tokens_fee_disabled() {
    let mut state = EnshrineTestState::new();
    let mut payments = PaymentsVec::new();
    let amount = BigUint::from(100000000000000000u128);
    let wegld_payment = EsdtTokenPayment::new(WEGLD_IDENTIFIER.into(), 0, amount.clone());
    let fungible_payment = EsdtTokenPayment::new(FUNGIBLE_TOKEN_ID.into(), 0, amount.clone());
    let crowd_payment = EsdtTokenPayment::new(CROWD_TOKEN_ID.into(), 0, amount.clone());
    let mut token_whitelist = MultiValueEncoded::new();
    token_whitelist.push(NFT_TOKEN_ID.into());

    payments.push(wegld_payment);
    payments.push(fungible_payment);
    payments.push(crowd_payment);

    state.setup_contracts(false, None, None);
    state.add_token_to_whitelist(token_whitelist);
    state.deposit(
        OWNER_ADDRESS,
        USER_ADDRESS,
        payments,
        OptionalValue::None,
        None,
    );

    let expected_amount = BigUint::from(ENSHRINE_BALANCE);

    let expected_balances = vec![
        MultiValue3::from((FUNGIBLE_TOKEN_ID, 0u64, expected_amount.clone())),
        MultiValue3::from((CROWD_TOKEN_ID, 0u64, expected_amount)),
    ];

    state
        .common_setup
        .check_account_multiple_esdts(OWNER_ADDRESS.to_address(), expected_balances);
}

#[test]
fn test_deposit_refund_non_whitelisted_tokens_fee_enabled() {
    let mut state = EnshrineTestState::new();
    let mut payments = PaymentsVec::new();
    let amount = BigUint::from(100000000000000000u128);
    let wegld_payment = EsdtTokenPayment::new(WEGLD_IDENTIFIER.into(), 0, amount.clone());
    let fungible_payment = EsdtTokenPayment::new(FUNGIBLE_TOKEN_ID.into(), 0, amount.clone());
    let crowd_payment = EsdtTokenPayment::new(CROWD_TOKEN_ID.into(), 0, amount.clone());
    let mut token_whitelist = MultiValueEncoded::new();
    token_whitelist.push(NFT_TOKEN_ID.into());

    payments.push(wegld_payment);
    payments.push(fungible_payment);
    payments.push(crowd_payment);

    let fee_amount_per_transfer = BigUint::from(100u32);
    let fee_amount_per_gas = BigUint::from(100u32);

    let fee_struct = state.setup_fee_struct(
        WEGLD_IDENTIFIER,
        &fee_amount_per_transfer,
        &fee_amount_per_gas,
    );

    state.setup_contracts(false, Some(&fee_struct), None);
    state.add_token_to_whitelist(token_whitelist);
    state.set_fee(Some(&fee_struct), None);
    state.deposit(
        OWNER_ADDRESS,
        USER_ADDRESS,
        payments,
        OptionalValue::None,
        None,
    );

    let expected_amount = BigUint::from(ENSHRINE_BALANCE);

    let expected_balances = vec![
        MultiValue3::from((FUNGIBLE_TOKEN_ID, 0u64, expected_amount.clone())),
        MultiValue3::from((CROWD_TOKEN_ID, 0u64, expected_amount)),
    ];

    state
        .common_setup
        .check_account_multiple_esdts(OWNER_ADDRESS.to_address(), expected_balances);
}
