use common_interactor::common_sovereign_interactor::CommonInteractorTrait;
use common_interactor::interactor_config::Config;
use common_test_setup::constants::{
    CROWD_TOKEN_ID, DEPOSIT_LOG, ISSUE_COST, NFT_TOKEN_ID, ONE_HUNDRED_THOUSAND,
    ONE_HUNDRED_TOKENS, ONE_THOUSAND_TOKENS, PREFIX_NFT_TOKEN_ID, TEN_TOKENS, TESTING_SC_ENDPOINT,
};
use error_messages::{
    BANNED_ENDPOINT_NAME, GAS_LIMIT_TOO_HIGH, NOTHING_TO_TRANSFER, NOT_ENOUGH_WEGLD_AMOUNT,
    ONLY_WEGLD_IS_ACCEPTED_AS_REGISTER_FEE, PAYMENT_DOES_NOT_COVER_FEE, TOO_MANY_TOKENS,
};
use multiversx_sc::imports::{MultiValue3, OptionalValue};
use multiversx_sc::types::{
    BigUint, EsdtTokenPayment, ManagedBuffer, ManagedVec, MultiValueEncoded, TokenIdentifier,
};
use multiversx_sc_snippets::imports::*;
use rust_interact::enshrine_esdt_safe::enshrine_esdt_safe_interactor::EnshrineEsdtSafeInteract;
use serial_test::serial;
use structs::aliases::{OptionalTransferData, PaymentsVec};
use structs::configs::EsdtSafeConfig;
use structs::fee::{FeeStruct, FeeType};
use structs::forge::ScArray;

/// ### TEST
/// E-ESDT_DEPLOY_OK
///
/// ### ACTION
/// Call 'setup_contracts()'
///
/// ### EXPECTED
/// Contracts are deployed successfully
#[tokio::test]
#[ignore]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deploy() {
    let mut chain_interactor =
        EnshrineEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    chain_interactor
        .deploy_contracts(
            false,
            None,
            None,
            vec![ScArray::ChainConfig, ScArray::EnshrineESDTSafe],
        )
        .await;
}

/// ### TEST
/// E-ESDT_REGISTER_FAIL
///
/// ### ACTION
/// Call 'register_tokens()' with invalid token as fee
///
/// ### EXPECTED
/// Error ONLY_WEGLD_IS_ACCEPTED_AS_REGISTER_FEE
#[tokio::test]
#[ignore]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_register_tokens_wrong_token_as_fee() {
    let mut chain_interactor =
        EnshrineEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    let token_vec = vec![
        TokenIdentifier::from(PREFIX_NFT_TOKEN_ID),
        TokenIdentifier::from(CROWD_TOKEN_ID),
    ];
    let payment_amount = BigUint::from(ISSUE_COST * token_vec.len() as u64);
    let payment = EsdtTokenPayment::new(
        chain_interactor.state.get_second_token_identifier(),
        0,
        payment_amount,
    );

    chain_interactor
        .deploy_contracts(
            false,
            None,
            None,
            vec![ScArray::ChainConfig, ScArray::EnshrineESDTSafe],
        )
        .await;

    chain_interactor
        .register_tokens(
            payment,
            token_vec,
            Some(ONLY_WEGLD_IS_ACCEPTED_AS_REGISTER_FEE),
        )
        .await;

    chain_interactor.check_user_balance_unchanged().await;
}

/// ### TEST
/// E-ESDT_REGISTER_OK
///
/// ### ACTION
/// Call 'register_tokens()' with valid payments
///
/// ### EXPECTED
/// Token is registered successfully
#[tokio::test]
#[ignore]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_register_tokens() {
    let mut chain_interactor =
        EnshrineEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    let token_vec = vec![
        TokenIdentifier::from(PREFIX_NFT_TOKEN_ID),
        TokenIdentifier::from(CROWD_TOKEN_ID),
    ];
    let payment_amount = BigUint::from(ISSUE_COST * token_vec.len() as u64);
    let payment = EsdtTokenPayment::new(
        chain_interactor.state.get_first_token_identifier(),
        0,
        payment_amount.clone(),
    );

    chain_interactor
        .deploy_contracts(
            false,
            None,
            None,
            vec![ScArray::ChainConfig, ScArray::EnshrineESDTSafe],
        )
        .await;

    chain_interactor
        .register_tokens(payment, token_vec, None)
        .await;

    let token = chain_interactor.state.get_first_token_id();
    chain_interactor
        .check_user_balance_after_deduction(token, payment_amount)
        .await;
}

/// ### TEST
/// E-ESDT_REGISTER_FAIL
///
/// ### ACTION
/// Call 'register_tokens()' with insufficient WEGLD amount
///
/// ### EXPECTED
/// Error NOT_ENOUGH_WEGLD_AMOUNT
#[tokio::test]
#[ignore]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_register_tokens_insufficient_wegld() {
    let mut chain_interactor =
        EnshrineEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    let token_vec = vec![
        TokenIdentifier::from(PREFIX_NFT_TOKEN_ID),
        TokenIdentifier::from(CROWD_TOKEN_ID),
    ];
    let payment_amount = BigUint::from(token_vec.len() as u64);
    let payment = EsdtTokenPayment::new(
        chain_interactor.state.get_first_token_identifier(),
        0,
        payment_amount,
    );

    chain_interactor
        .deploy_contracts(
            false,
            None,
            None,
            vec![ScArray::ChainConfig, ScArray::EnshrineESDTSafe],
        )
        .await;

    chain_interactor
        .register_tokens(payment, token_vec, Some(NOT_ENOUGH_WEGLD_AMOUNT))
        .await;
    chain_interactor.check_user_balance_unchanged().await;
}

/// ### TEST
/// E-ESDT_DEPOSIT_OK
///
/// ### ACTION
/// Call 'deposit()' with valid payments
///
/// ### EXPECTED
/// Deposit is executed successfully
#[tokio::test]
#[ignore]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_no_fee() {
    let mut chain_interactor =
        EnshrineEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    let amount = BigUint::from(ONE_HUNDRED_THOUSAND);
    let wegld_payment = EsdtTokenPayment::new(
        chain_interactor.state.get_first_token_identifier(),
        0,
        amount.clone(),
    );
    let mut payments = PaymentsVec::new();

    payments.push(wegld_payment);

    chain_interactor
        .deploy_contracts(
            false,
            None,
            None,
            vec![ScArray::ChainConfig, ScArray::EnshrineESDTSafe],
        )
        .await;

    chain_interactor
        .deposit(
            payments,
            chain_interactor.user_address.clone(),
            OptionalValue::None,
            None,
            Some(DEPOSIT_LOG),
        )
        .await;
}

/// ### TEST
/// E-ESDT_DEPOSIT_FAIL
///
/// ### ACTION
/// Call 'deposit()' with no payments
///
/// ### EXPECTED
/// Error NOTHING_TO_TRANSFER
#[tokio::test]
#[ignore]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_token_nothing_to_transfer_fee_disabled() {
    let mut chain_interactor =
        EnshrineEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    let payments = PaymentsVec::new();

    chain_interactor
        .deploy_contracts(
            false,
            None,
            None,
            vec![ScArray::ChainConfig, ScArray::EnshrineESDTSafe],
        )
        .await;

    chain_interactor
        .deposit(
            payments,
            chain_interactor.user_address.clone(),
            OptionalValue::None,
            Some(NOTHING_TO_TRANSFER),
            None,
        )
        .await;

    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor
        .check_enshrine_esdt_safe_balance_is_empty()
        .await;
}

/// ### TEST
/// E-ESDT_DEPOSIT_FAIL
///
/// ### ACTION
/// Call 'deposit()' with too many payments
///
/// ### EXPECTED
/// Error TOO_MANY_TOKENS
#[tokio::test]
#[ignore]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_max_transfers_exceeded() {
    let mut chain_interactor =
        EnshrineEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let amount = BigUint::from(TEN_TOKENS);
    let wegld_payment = EsdtTokenPayment::new(
        chain_interactor.state.get_first_token_identifier(),
        0,
        amount,
    );
    let mut payments = PaymentsVec::new();
    payments.extend(vec![wegld_payment; 11]);

    chain_interactor
        .deploy_contracts(
            false,
            None,
            None,
            vec![ScArray::ChainConfig, ScArray::EnshrineESDTSafe],
        )
        .await;

    chain_interactor
        .deposit(
            payments,
            chain_interactor.user_address.clone(),
            OptionalValue::None,
            Some(TOO_MANY_TOKENS),
            None,
        )
        .await;

    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor
        .check_enshrine_esdt_safe_balance_is_empty()
        .await;
}

/// ### TEST
/// E-ESDT_DEPOSIT_OK
///
/// ### ACTION
/// Call 'deposit()' with no transfer data
///
/// ### EXPECTED
/// Deposit is executed successfully
#[tokio::test]
#[ignore]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_no_transfer_data() {
    let mut chain_interactor =
        EnshrineEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    let amount = BigUint::from(ONE_HUNDRED_TOKENS);
    let wegld_payment = EsdtTokenPayment::new(
        chain_interactor.state.get_first_token_identifier(),
        0,
        amount.clone(),
    );
    let fungible_payment = EsdtTokenPayment::new(
        chain_interactor.state.get_second_token_identifier(),
        0,
        amount.clone(),
    );
    let mut payments = PaymentsVec::new();
    let mut tokens_whitelist = MultiValueVec::new();
    tokens_whitelist.push(chain_interactor.state.get_second_token_identifier());

    payments.push(wegld_payment);
    payments.push(fungible_payment);

    let fee_amount_per_transfer = BigUint::from(100u32);
    let fee_amount_per_gas = BigUint::from(100u32);

    let fee_struct = FeeStruct {
        base_token: chain_interactor.state.get_first_token_identifier(),
        fee_type: FeeType::Fixed {
            token: chain_interactor.state.get_first_token_identifier(),
            per_transfer: fee_amount_per_transfer.clone(),
            per_gas: fee_amount_per_gas,
        },
    };

    chain_interactor
        .deploy_contracts(
            false,
            Some(fee_struct),
            None,
            vec![ScArray::ChainConfig, ScArray::EnshrineESDTSafe],
        )
        .await;

    chain_interactor
        .add_tokens_to_whitelist(tokens_whitelist)
        .await;

    chain_interactor
        .deposit(
            payments,
            chain_interactor.user_address.clone(),
            OptionalValue::None,
            None,
            Some(DEPOSIT_LOG),
        )
        .await;
}

/// ### TEST
/// E-ESDT_DEPOSIT_FAIL
///
/// ### ACTION
/// Call 'deposit()' with gas limit too high in transfer data
///
/// ### EXPECTED
/// Error GAS_LIMIT_TOO_HIGH
#[tokio::test]
#[ignore]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_with_transfer_data_gas_limit_too_high() {
    let mut chain_interactor =
        EnshrineEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    let amount = BigUint::from(ONE_HUNDRED_THOUSAND);
    let wegld_payment = EsdtTokenPayment::new(
        chain_interactor.state.get_first_token_identifier(),
        0,
        amount.clone(),
    );
    let second_payment = EsdtTokenPayment::new(
        chain_interactor.state.get_second_token_identifier(),
        0,
        amount,
    );
    let mut payments = PaymentsVec::new();
    let gas_limit = 1_000_000_000_000_000_000u64;
    let function = ManagedBuffer::from(TESTING_SC_ENDPOINT);
    let arg = ManagedBuffer::from("arg");
    let mut args = ManagedVec::new();
    args.push(arg);

    let args_encoded = MultiValueEncoded::from(args);

    let transfer_data =
        OptionalTransferData::Some(MultiValue3::from((gas_limit, function, args_encoded)));

    payments.push(wegld_payment);
    payments.push(second_payment);

    chain_interactor
        .deploy_contracts(
            false,
            None,
            None,
            vec![ScArray::ChainConfig, ScArray::EnshrineESDTSafe],
        )
        .await;

    chain_interactor
        .deposit(
            payments,
            chain_interactor.user_address.clone(),
            transfer_data,
            Some(GAS_LIMIT_TOO_HIGH),
            None,
        )
        .await;

    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor
        .check_enshrine_esdt_safe_balance_is_empty()
        .await;
}

/// ### TEST
/// E-ESDT_DEPOSIT_FAIL
///
/// ### ACTION
/// Call 'deposit()' with banned endpoint in transfer data
///
/// ### EXPECTED
/// Error BANNED_ENDPOINT_NAME
#[tokio::test]
#[ignore]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_with_transfer_data_banned_endpoint() {
    let mut chain_interactor =
        EnshrineEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    let amount = BigUint::from(ONE_HUNDRED_THOUSAND);
    let wegld_payment = EsdtTokenPayment::new(
        chain_interactor.state.get_first_token_identifier(),
        0,
        amount.clone(),
    );
    let crowd_payment = EsdtTokenPayment::new(
        chain_interactor.state.get_second_token_identifier(),
        0,
        amount,
    );
    let mut payments = PaymentsVec::new();
    let gas_limit = 1_000_000_000;
    let banned_endpoint = ManagedBuffer::from("some_function");
    let arg = ManagedBuffer::from("arg");
    let mut args = ManagedVec::new();
    args.push(arg);

    let args_encoded = MultiValueEncoded::from(args);

    let transfer_data = OptionalTransferData::Some(MultiValue3::from((
        gas_limit,
        banned_endpoint.clone(),
        args_encoded,
    )));
    payments.push(wegld_payment);
    payments.push(crowd_payment);

    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        300_000_000_000,
        ManagedVec::from(vec![banned_endpoint]),
        ManagedVec::new(),
    );

    chain_interactor
        .deploy_contracts(
            false,
            None,
            Some(config),
            vec![ScArray::ChainConfig, ScArray::EnshrineESDTSafe],
        )
        .await;

    chain_interactor
        .deposit(
            payments,
            chain_interactor.user_address.clone(),
            transfer_data,
            Some(BANNED_ENDPOINT_NAME),
            None,
        )
        .await;
    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor
        .check_enshrine_esdt_safe_balance_is_empty()
        .await;
}

/// ### TEST
/// E-ESDT_DEPOSIT_OK
///
/// ### ACTION
/// Call 'deposit()' with transfer data and fee
///
/// ### EXPECTED
/// Deposit is executed successfully
#[tokio::test]
#[ignore]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_with_transfer_data_enough_for_fee() {
    let mut chain_interactor =
        EnshrineEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    let amount = BigUint::from(ONE_THOUSAND_TOKENS);
    let wegld_payment = EsdtTokenPayment::new(
        chain_interactor.state.get_first_token_identifier(),
        0,
        amount.clone(),
    );
    let fungible_payment = EsdtTokenPayment::new(
        chain_interactor.state.get_second_token_identifier(),
        0,
        amount.clone(),
    );
    let mut payments = PaymentsVec::new();
    let gas_limit = 10_000_000;
    let function = ManagedBuffer::from("some_function");
    let arg = ManagedBuffer::from("arg");
    let mut args = ManagedVec::new();
    args.push(arg);

    let args_encoded = MultiValueEncoded::from(args);

    let transfer_data =
        OptionalTransferData::Some(MultiValue3::from((gas_limit, function, args_encoded)));

    payments.push(wegld_payment);
    payments.push(fungible_payment);

    let fee_amount_per_transfer = BigUint::from(100u32);
    let fee_amount_per_gas = BigUint::from(100u32);

    let fee_struct = FeeStruct {
        base_token: chain_interactor.state.get_first_token_identifier(),
        fee_type: FeeType::Fixed {
            token: chain_interactor.state.get_first_token_identifier(),
            per_transfer: fee_amount_per_transfer.clone(),
            per_gas: fee_amount_per_gas.clone(),
        },
    };

    chain_interactor
        .deploy_contracts(
            false,
            Some(fee_struct),
            None,
            vec![ScArray::ChainConfig, ScArray::EnshrineESDTSafe],
        )
        .await;

    chain_interactor
        .deposit(
            payments,
            chain_interactor.user_address.clone(),
            transfer_data,
            None,
            Some(DEPOSIT_LOG),
        )
        .await;
}

/// ### TEST
/// E-ESDT_DEPOSIT_FAIL
///
/// ### ACTION
/// Call 'deposit()' with transfer data and not enough fee tokens
///
/// ### EXPECTED
/// Error PAYMENT_DOES_NOT_COVER_FEE
#[tokio::test]
#[ignore]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_with_transfer_data_not_enough_for_fee() {
    let mut chain_interactor =
        EnshrineEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    let amount = BigUint::from(ONE_HUNDRED_THOUSAND);
    let wegld_payment = EsdtTokenPayment::new(
        chain_interactor.state.get_first_token_identifier(),
        0,
        amount.clone(),
    );
    let fungible_payment = EsdtTokenPayment::new(
        chain_interactor.state.get_second_token_identifier(),
        0,
        amount.clone(),
    );
    let mut payments = PaymentsVec::new();
    let gas_limit = 10000000;
    let function = ManagedBuffer::from("some_function");
    let arg = ManagedBuffer::from("arg");
    let mut args = ManagedVec::new();
    args.push(arg);

    let args_encoded = MultiValueEncoded::from(args);

    let transfer_data =
        OptionalTransferData::Some(MultiValue3::from((gas_limit, function, args_encoded)));

    payments.push(wegld_payment);
    payments.push(fungible_payment);

    let fee_amount_per_transfer = BigUint::from(100u32);
    let fee_amount_per_gas = BigUint::from(100u32);

    let fee_struct = FeeStruct {
        base_token: chain_interactor.state.get_first_token_identifier(),
        fee_type: FeeType::Fixed {
            token: chain_interactor.state.get_first_token_identifier(),
            per_transfer: fee_amount_per_transfer,
            per_gas: fee_amount_per_gas,
        },
    };

    chain_interactor
        .deploy_contracts(
            false,
            Some(fee_struct),
            None,
            vec![ScArray::ChainConfig, ScArray::EnshrineESDTSafe],
        )
        .await;

    chain_interactor
        .deposit(
            payments,
            chain_interactor.user_address.clone(),
            transfer_data,
            Some(PAYMENT_DOES_NOT_COVER_FEE),
            None,
        )
        .await;

    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor
        .check_enshrine_esdt_safe_balance_is_empty()
        .await;
}

/// ### TEST
/// E-ESDT_DEPOSIT_OK
///
/// ### ACTION
/// Call 'deposit()' with non whitelisted tokens
///
/// ### EXPECTED
/// Deposit is executed successfully and the tokens are refunded
#[tokio::test]
#[ignore]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_refund_non_whitelisted_tokens_fee_disabled() {
    let mut chain_interactor =
        EnshrineEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    let mut payments = PaymentsVec::new();
    let amount = BigUint::from(100000000000000000u128);
    let wegld_payment = EsdtTokenPayment::new(
        chain_interactor.state.get_first_token_identifier(),
        0,
        amount.clone(),
    );
    let fungible_payment = EsdtTokenPayment::new(
        chain_interactor.state.get_second_token_identifier(),
        0,
        amount.clone(),
    );
    let mut token_whitelist = MultiValueVec::new();
    token_whitelist.push(NFT_TOKEN_ID.into());

    payments.push(wegld_payment);
    payments.push(fungible_payment);

    chain_interactor
        .deploy_contracts(
            false,
            None,
            None,
            vec![ScArray::ChainConfig, ScArray::EnshrineESDTSafe],
        )
        .await;
    chain_interactor
        .add_tokens_to_whitelist(token_whitelist)
        .await;

    chain_interactor
        .deposit(
            payments,
            chain_interactor.user_address.clone(),
            OptionalValue::None,
            None,
            Some(DEPOSIT_LOG),
        )
        .await;
    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor
        .check_enshrine_esdt_safe_balance_is_empty()
        .await;
}

/// ### TEST
/// E-ESDT_DEPOSIT_OK
///
/// ### ACTION
/// Call 'deposit()' with non whitelisted tokens and fee enabled
///
/// ### EXPECTED
/// Deposit is executed successfully and all the tokens are refunded
#[tokio::test]
#[ignore]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_refund_non_whitelisted_tokens_fee_enabled() {
    let mut chain_interactor =
        EnshrineEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    let mut payments = PaymentsVec::new();
    let amount = BigUint::from(ONE_THOUSAND_TOKENS);
    let wegld_payment = EsdtTokenPayment::new(
        chain_interactor.state.get_first_token_identifier(),
        0,
        amount.clone(),
    );
    let fungible_payment = EsdtTokenPayment::new(
        chain_interactor.state.get_second_token_identifier(),
        0,
        amount.clone(),
    );
    let mut token_whitelist = MultiValueVec::new();
    token_whitelist.push(NFT_TOKEN_ID.into());

    payments.push(wegld_payment);
    payments.push(fungible_payment);

    let fee_amount_per_transfer = BigUint::from(100u32);
    let fee_amount_per_gas = BigUint::from(100u32);

    let fee_struct = FeeStruct {
        base_token: chain_interactor.state.get_first_token_identifier(),
        fee_type: FeeType::Fixed {
            token: chain_interactor.state.get_first_token_identifier(),
            per_transfer: fee_amount_per_transfer.clone(),
            per_gas: fee_amount_per_gas,
        },
    };

    chain_interactor
        .deploy_contracts(
            false,
            Some(fee_struct),
            None,
            vec![ScArray::ChainConfig, ScArray::EnshrineESDTSafe],
        )
        .await;
    chain_interactor
        .add_tokens_to_whitelist(token_whitelist)
        .await;

    chain_interactor
        .deposit(
            payments,
            chain_interactor.user_address.clone(),
            OptionalValue::None,
            None,
            Some(DEPOSIT_LOG),
        )
        .await;

    chain_interactor.check_user_balance_unchanged().await;
    chain_interactor
        .check_enshrine_esdt_safe_balance_is_empty()
        .await;
}
