use common_interactor::common_sovereign_interactor::CommonInteractorTrait;
use common_interactor::constants::{ONE_HUNDRED_TOKENS, ONE_THOUSAND_TOKENS, TEN_TOKENS};
use common_interactor::interactor_config::Config;
use common_test_setup::constants::{
    CROWD_TOKEN_ID, ISSUE_COST, MVX_TO_SOV_TOKEN_STORAGE_KEY, OPERATION_HASH_STATUS_STORAGE_KEY,
    SOV_TOKEN, SOV_TO_MVX_TOKEN_STORAGE_KEY, TOKEN_TICKER,
};
use common_test_setup::RegisterTokenArgs;
use error_messages::{
    BANNED_ENDPOINT_NAME, GAS_LIMIT_TOO_HIGH, INVALID_TYPE, NOTHING_TO_TRANSFER,
    PAYMENT_DOES_NOT_COVER_FEE, SETUP_PHASE_NOT_COMPLETED, TOO_MANY_TOKENS,
};
use header_verifier::OperationHashStatus;
use multiversx_sc_snippets::multiversx_sc_scenario::multiversx_chain_vm::crypto_functions::sha256;
use multiversx_sc_snippets::{hex, imports::*};
use rust_interact::mvx_esdt_safe::mvx_esdt_safe_interactor_main::MvxEsdtSafeInteract;
use serial_test::serial;
use structs::aliases::PaymentsVec;
use structs::configs::EsdtSafeConfig;
use structs::fee::{FeeStruct, FeeType};
use structs::forge::ScArray;
use structs::operation::{Operation, OperationData, OperationEsdtPayment, TransferData};

/// ### TEST
/// M-ESDT_ISSUE_OK
///
/// ### ACTION
/// Issue and mint all types of tokens to the wallet address
///
/// ### EXPECTED
/// All the tokens are minted to the wallet address
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_issue_tokens() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    let wallet_address = chain_interactor.wallet_address().clone();
    let user_address = chain_interactor.user_address.clone();
    let first_token_id = chain_interactor.state.get_first_token_id().clone();

    chain_interactor
        .issue_and_mint_the_remaining_types_of_tokens()
        .await;

    chain_interactor
        .interactor()
        .tx()
        .from(wallet_address)
        .to(user_address.clone())
        .single_esdt(&first_token_id, 0u64, &BigUint::from(ONE_THOUSAND_TOKENS))
        .run()
        .await;

    let expected_token =
        vec![chain_interactor.thousand_tokens(chain_interactor.state.get_first_token_id_string())];

    chain_interactor
        .check_address_balance(&Bech32Address::from(user_address), expected_token)
        .await;
}

/// ### TEST
/// M-ESDT_DEP_FAIL
///
/// ### ACTION
/// Call 'deposit()' with empty payments_vec and no transfer_data
///
/// ### EXPECTED
/// Error NOTHING_TO_TRANSFER
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_nothing_to_transfer_no_fee() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    chain_interactor
        .deploy_contracts(
            OptionalValue::None,
            OptionalValue::Some(EsdtSafeConfig::default_config()),
            None,
            vec![ScArray::ChainConfig, ScArray::ESDTSafe],
        )
        .await;

    chain_interactor
        .deposit(
            chain_interactor.user_address.clone(),
            OptionalValue::None,
            ManagedVec::new(),
            Some(NOTHING_TO_TRANSFER),
            None,
        )
        .await;

    chain_interactor.check_wallet_balance().await;
    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty()
        .await;
    chain_interactor.check_fee_market_balance_is_empty().await;
}

/// ### TEST
/// M-ESDT_DEP_FAIL
///
/// ### ACTION
/// Call 'deposit()' with too many tokens in payments_vec
///
/// ### EXPECTED
/// Error TOO_MANY_TOKENS
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_too_many_tokens_no_fee() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    chain_interactor
        .deploy_contracts(
            OptionalValue::None,
            OptionalValue::Some(EsdtSafeConfig::default_config()),
            None,
            vec![ScArray::ChainConfig, ScArray::ESDTSafe],
        )
        .await;

    let esdt_token_payment = EsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_first_token_id(),
        0,
        BigUint::from(1u64),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment; 11]);

    chain_interactor
        .deposit(
            chain_interactor.user_address.clone(),
            OptionalValue::None,
            payments_vec,
            Some(TOO_MANY_TOKENS),
            None,
        )
        .await;

    chain_interactor.check_wallet_balance().await;
    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty()
        .await;
    chain_interactor.check_fee_market_balance_is_empty().await;
}

/// ### TEST
/// M-ESDT_DEP_FAIL
///
/// ### ACTION
/// Call 'deposit()' with no transfer_data and no payments_vec
///
/// ### EXPECTED
/// Error NOTHING_TO_TRANSFER
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_no_transfer_data_no_fee() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let wallet_address = chain_interactor.wallet_address().clone();

    chain_interactor
        .deploy_contracts(
            OptionalValue::None,
            OptionalValue::Some(EsdtSafeConfig::default_config()),
            None,
            vec![ScArray::ChainConfig, ScArray::ESDTSafe],
        )
        .await;

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_first_token_id(),
        0,
        BigUint::from(ONE_HUNDRED_TOKENS),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_second_token_id(),
        0,
        BigUint::from(ONE_HUNDRED_TOKENS),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one, esdt_token_payment_two]);

    chain_interactor
        .deposit(
            chain_interactor.user_address.clone(),
            OptionalValue::None,
            payments_vec,
            None,
            Some("deposit"),
        )
        .await;

    let expected_tokens_mvx_esdt_safe = vec![
        chain_interactor.custom_amount_tokens(
            chain_interactor.state.get_first_token_id_string(),
            ONE_HUNDRED_TOKENS,
        ),
        chain_interactor.custom_amount_tokens(
            chain_interactor.state.get_second_token_id_string(),
            ONE_HUNDRED_TOKENS,
        ),
    ];

    chain_interactor
        .check_address_balance(
            &chain_interactor
                .state
                .current_mvx_esdt_safe_contract_address()
                .clone(),
            expected_tokens_mvx_esdt_safe,
        )
        .await;

    let expected_tokens_wallet = vec![
        chain_interactor.custom_amount_tokens(
            chain_interactor.state.get_first_token_id_string(),
            ONE_THOUSAND_TOKENS - ONE_HUNDRED_TOKENS,
        ),
        chain_interactor.custom_amount_tokens(
            chain_interactor.state.get_second_token_id_string(),
            ONE_THOUSAND_TOKENS - ONE_HUNDRED_TOKENS,
        ),
        chain_interactor.thousand_tokens(chain_interactor.state.get_fee_token_id_string()),
    ];
    chain_interactor
        .check_address_balance(&Bech32Address::from(wallet_address), expected_tokens_wallet)
        .await;

    chain_interactor.check_fee_market_balance_is_empty().await;
}

/// ### TEST
/// M-ESDT_DEP_FAIL
///
/// ### ACTION
/// Call 'deposit()' with gas limit too high in transfer_data
///
/// ### EXPECTED
/// Error GAS_LIMIT_TOO_HIGH
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn deposit_gas_limit_too_high_no_fee() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        1,
        ManagedVec::new(),
        ManagedVec::new(),
    );

    chain_interactor
        .deploy_contracts(
            OptionalValue::None,
            OptionalValue::Some(config),
            None,
            vec![ScArray::ChainConfig, ScArray::ESDTSafe],
        )
        .await;

    chain_interactor.deploy_testing_sc().await;

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_first_token_id(),
        0,
        BigUint::from(ONE_HUNDRED_TOKENS),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_second_token_id(),
        0,
        BigUint::from(ONE_HUNDRED_TOKENS),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one, esdt_token_payment_two]);

    let gas_limit = 2u64;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args = MultiValueEncoded::from(ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(
        vec![ManagedBuffer::from("1")],
    ));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    chain_interactor
        .deposit(
            chain_interactor.user_address.clone(),
            OptionalValue::Some(transfer_data),
            payments_vec,
            Some(GAS_LIMIT_TOO_HIGH),
            None,
        )
        .await;

    chain_interactor.check_wallet_balance().await;
    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty()
        .await;
    chain_interactor.check_fee_market_balance_is_empty().await;
}

/// ### TEST
/// M-ESDT_DEP_FAIL
///
/// ### ACTION
/// Call 'deposit()' with banned endpoint name in transfer_data
///
/// ### EXPECTED
/// Error BANNED_ENDPOINT_NAME
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn deposit_endpoint_banned_no_fee() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::from(vec![ManagedBuffer::from("hello")]),
        ManagedVec::new(),
    );

    chain_interactor
        .deploy_contracts(
            OptionalValue::None,
            OptionalValue::Some(config),
            None,
            vec![ScArray::ChainConfig, ScArray::ESDTSafe],
        )
        .await;

    chain_interactor.deploy_testing_sc().await;

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_first_token_id(),
        0,
        BigUint::from(ONE_HUNDRED_TOKENS),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_second_token_id(),
        0,
        BigUint::from(ONE_HUNDRED_TOKENS),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one, esdt_token_payment_two]);

    let gas_limit = 2u64;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args = MultiValueEncoded::from(ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(
        vec![ManagedBuffer::from("1")],
    ));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    chain_interactor
        .deposit(
            chain_interactor.user_address.clone(),
            OptionalValue::Some(transfer_data),
            payments_vec,
            Some(BANNED_ENDPOINT_NAME),
            None,
        )
        .await;

    chain_interactor.check_wallet_balance().await;
    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty()
        .await;
    chain_interactor.check_fee_market_balance_is_empty().await;
}

/// ### TEST
/// M-ESDT_DEP_OK
///
/// ### ACTION
/// Call 'deposit()' with transfer data and valid payment
///
/// ### EXPECTED
/// USER's balance is updated
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn deposit_fee_enabled() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::new(),
        ManagedVec::new(),
    );

    let per_transfer = BigUint::from(1u64);
    let per_gas = BigUint::from(1u64);
    let fee_token = chain_interactor.state.get_fee_token_id();

    let fee = FeeStruct {
        base_token: fee_token.clone(),
        fee_type: FeeType::Fixed {
            token: fee_token.clone(),
            per_transfer: per_transfer.clone(),
            per_gas,
        },
    };

    chain_interactor
        .deploy_contracts(
            OptionalValue::None,
            OptionalValue::Some(config),
            Some(fee),
            vec![ScArray::ChainConfig, ScArray::ESDTSafe],
        )
        .await;

    chain_interactor.deploy_testing_sc().await;

    let fee_amount = BigUint::from(ONE_HUNDRED_TOKENS);

    let fee_payment = EsdtTokenPayment::<StaticApi>::new(fee_token, 0, fee_amount);

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_first_token_id(),
        0,
        BigUint::from(ONE_HUNDRED_TOKENS),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_second_token_id(),
        0,
        BigUint::from(ONE_HUNDRED_TOKENS),
    );

    let payments_vec = PaymentsVec::from(vec![
        fee_payment,
        esdt_token_payment_one,
        esdt_token_payment_two,
    ]);

    let gas_limit = 1000u64;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args = MultiValueEncoded::from(ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(
        vec![ManagedBuffer::from("1")],
    ));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    chain_interactor
        .deposit(
            chain_interactor.user_address.clone(),
            OptionalValue::Some(transfer_data),
            payments_vec.clone(),
            None,
            Some("deposit"),
        )
        .await;

    let expected_mvx_esdt_safe_tokens = vec![
        chain_interactor.hundred_tokens(chain_interactor.state.get_first_token_id_string()),
        chain_interactor.hundred_tokens(chain_interactor.state.get_second_token_id_string()),
    ];
    chain_interactor
        .check_address_balance(
            &chain_interactor
                .state
                .current_mvx_esdt_safe_contract_address()
                .clone(),
            expected_mvx_esdt_safe_tokens,
        )
        .await;

    let expected_fee_market_token_amount =
        BigUint::from(gas_limit) + BigUint::from(payments_vec.len() - 1) * per_transfer.clone();

    let expected_fee_market_tokens = vec![
        (chain_interactor.custom_amount_tokens(
            chain_interactor.state.get_fee_token_id_string(),
            expected_fee_market_token_amount.clone(),
        )),
    ];
    chain_interactor
        .check_address_balance(
            &chain_interactor.state.current_fee_market_address().clone(),
            expected_fee_market_tokens,
        )
        .await;

    let expected_remaining_fee_tokens =
        BigUint::from(ONE_THOUSAND_TOKENS) - expected_fee_market_token_amount;
    let expected_tokens_wallet = vec![
        chain_interactor.custom_amount_tokens(
            chain_interactor.state.get_first_token_id_string(),
            ONE_THOUSAND_TOKENS - ONE_HUNDRED_TOKENS,
        ),
        chain_interactor.custom_amount_tokens(
            chain_interactor.state.get_second_token_id_string(),
            ONE_THOUSAND_TOKENS - ONE_HUNDRED_TOKENS,
        ),
        chain_interactor.custom_amount_tokens(
            chain_interactor.state.get_fee_token_id_string(),
            expected_remaining_fee_tokens,
        ),
    ];
    chain_interactor
        .check_address_balance(
            &Bech32Address::from(chain_interactor.wallet_address().clone()),
            expected_tokens_wallet,
        )
        .await
}

/// ### TEST
/// M-ESDT_DEP_OK
///
/// ### ACTION
/// Call 'deposit()' with transfer data only and no payments
///
/// ### EXPECTED
/// The endpoint is called in the testing smart contract
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn deposit_only_transfer_data_no_fee() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::new(),
        ManagedVec::new(),
    );

    chain_interactor
        .deploy_contracts(
            OptionalValue::None,
            OptionalValue::Some(config),
            None,
            vec![ScArray::ChainConfig, ScArray::ESDTSafe],
        )
        .await;

    chain_interactor.deploy_testing_sc().await;

    let gas_limit = 1000u64;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args = MultiValueEncoded::from(ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(
        vec![ManagedBuffer::from("1")],
    ));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    chain_interactor
        .deposit(
            chain_interactor.user_address.clone(),
            OptionalValue::Some(transfer_data),
            ManagedVec::new(),
            None,
            Some("scCall"),
        )
        .await;

    chain_interactor.check_wallet_balance().await;
    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty()
        .await;
    chain_interactor.check_fee_market_balance_is_empty().await;
}

/// ### TEST
/// M-ESDT_DEP_FAIL
///
/// ### ACTION
/// Call 'deposit()' with transfer data and payment not enough for fee
///
/// ### EXPECTED
/// Error PAYMENT_DOES_NOT_COVER_FEE
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn deposit_payment_does_not_cover_fee() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::new(),
        ManagedVec::new(),
    );

    let per_transfer = BigUint::from(1u64);
    let per_gas = BigUint::from(1u64);

    let fee = FeeStruct {
        base_token: chain_interactor.state.get_fee_token_id(),
        fee_type: FeeType::Fixed {
            token: chain_interactor.state.get_fee_token_id(),
            per_transfer,
            per_gas,
        },
    };

    chain_interactor
        .deploy_contracts(
            OptionalValue::None,
            OptionalValue::Some(config),
            Some(fee),
            vec![ScArray::ChainConfig, ScArray::ESDTSafe],
        )
        .await;

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_first_token_id(),
        0,
        BigUint::from(ONE_HUNDRED_TOKENS),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_second_token_id(),
        0,
        BigUint::from(ONE_HUNDRED_TOKENS),
    );

    let fee_payment = EsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_fee_token_id(),
        0,
        BigUint::from(1u64),
    );

    let payments_vec = PaymentsVec::from(vec![
        fee_payment,
        esdt_token_payment_one,
        esdt_token_payment_two,
    ]);

    let gas_limit = 10_000u64;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args = MultiValueEncoded::from(ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(
        vec![ManagedBuffer::from("1")],
    ));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    chain_interactor
        .deposit(
            chain_interactor.user_address.clone(),
            OptionalValue::Some(transfer_data),
            payments_vec,
            Some(PAYMENT_DOES_NOT_COVER_FEE),
            None,
        )
        .await;

    chain_interactor.check_wallet_balance().await;
    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty()
        .await;
    chain_interactor.check_fee_market_balance_is_empty().await;
}

#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_deposit_refund() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let wallet_address = chain_interactor.wallet_address().clone();

    let config = EsdtSafeConfig::new(
        ManagedVec::from(vec![TokenIdentifier::from(CROWD_TOKEN_ID)]),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::new(),
        ManagedVec::new(),
    );

    let per_transfer = BigUint::from(100u64);
    let per_gas = BigUint::from(1u64);

    let fee = FeeStruct {
        base_token: chain_interactor.state.get_fee_token_id(),
        fee_type: FeeType::Fixed {
            token: chain_interactor.state.get_fee_token_id(),
            per_transfer,
            per_gas,
        },
    };

    chain_interactor
        .deploy_contracts(
            SovereignConfig::default_config(),
            OptionalValue::Some(config),
            Some(fee),
        )
        .await;

    let fee_amount = BigUint::from(ONE_THOUSAND_TOKENS);

    let fee_payment = EsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_fee_token_id(),
        0,
        fee_amount.clone(),
    );

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_first_token_id(),
        0,
        BigUint::from(ONE_THOUSAND_TOKENS),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        chain_interactor.state.get_second_token_id(),
        0,
        BigUint::from(ONE_THOUSAND_TOKENS),
    );

    let payments_vec = PaymentsVec::from(vec![
        fee_payment,
        esdt_token_payment_one,
        esdt_token_payment_two,
    ]);

    let gas_limit = 1;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        MultiValueEncoded::<StaticApi, ManagedBuffer<StaticApi>>::from(ManagedVec::from(vec![
            ManagedBuffer::from("1"),
        ]));

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    chain_interactor
        .deposit(
            chain_interactor.user_address.clone(),
            OptionalValue::Some(transfer_data),
            payments_vec.clone(),
            None,
            Some("deposit"),
        )
        .await;

    let expected_tokens_wallet = vec![
        chain_interactor.thousand_tokens(chain_interactor.state.get_first_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_second_token_id_string()),
        chain_interactor.custom_amount_tokens(
            chain_interactor.state.get_fee_token_id_string(),
            ONE_THOUSAND_TOKENS - gas_limit as u128,
        ),
    ];
    chain_interactor
        .check_address_balance(&Bech32Address::from(wallet_address), expected_tokens_wallet)
        .await;

    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty()
        .await;

    let expected_tokens_fee_market = vec![chain_interactor
        .custom_amount_tokens(chain_interactor.state.get_fee_token_id_string(), gas_limit)];
    chain_interactor
        .check_address_balance(
            &chain_interactor.state.current_fee_market_address().clone(),
            expected_tokens_fee_market,
        )
        .await;
}

/// ### TEST
/// M-ESDT_REG_FAIL
///
/// ### ACTION
/// Call 'register_token()' with invalid token type
///
/// ### EXPECTED
/// Error CANNOT_REGISTER_TOKEN
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn register_token_invalid_type_token() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    chain_interactor
        .deploy_chain_config(OptionalValue::None)
        .await;

    let contracts_array =
        chain_interactor.get_contract_info_struct_for_sc_type(vec![ScArray::ChainConfig]);

    chain_interactor
        .deploy_header_verifier(contracts_array)
        .await;

    chain_interactor
        .deploy_mvx_esdt_safe(OptionalValue::Some(EsdtSafeConfig::default_config()))
        .await;

    let sov_token_id = TokenIdentifier::from_esdt_bytes(SOV_TOKEN.as_str());
    let token_type = EsdtTokenType::Invalid;
    let token_display_name = "SOVEREIGN";
    let num_decimals = 18;
    let token_ticker = TOKEN_TICKER;
    let egld_payment = BigUint::from(ISSUE_COST);

    chain_interactor
        .register_token(
            RegisterTokenArgs {
                sov_token_id,
                token_type,
                token_display_name,
                token_ticker,
                num_decimals,
            },
            egld_payment,
            Some(INVALID_TYPE),
        )
        .await;

    let key = hex::encode(MVX_TO_SOV_TOKEN_STORAGE_KEY);
    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .current_mvx_esdt_safe_contract_address()
                .clone()
                .to_address(),
            key.as_str(),
            None,
        )
        .await;
}

/// ### TEST
/// M-ESDT_REG_OK
///
/// ### ACTION
/// Call 'register_token()' with valid token id and type
///
/// ### EXPECTED
/// The token is registered
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn register_token_fungible_token() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    chain_interactor
        .deploy_chain_config(OptionalValue::None)
        .await;

    let contracts_array =
        chain_interactor.get_contract_info_struct_for_sc_type(vec![ScArray::ChainConfig]);

    chain_interactor
        .deploy_header_verifier(contracts_array)
        .await;

    chain_interactor
        .deploy_mvx_esdt_safe(OptionalValue::Some(EsdtSafeConfig::default_config()))
        .await;

    let sov_token_id = TokenIdentifier::from_esdt_bytes(SOV_TOKEN.as_str());
    let token_type = EsdtTokenType::Fungible;
    let token_display_name = "GREEN";
    let num_decimals = 18;
    let token_ticker = TOKEN_TICKER;
    let egld_payment = BigUint::from(ISSUE_COST);

    chain_interactor
        .register_token(
            RegisterTokenArgs {
                sov_token_id,
                token_type,
                token_display_name,
                token_ticker,
                num_decimals,
            },
            egld_payment,
            None,
        )
        .await;

    let encoded_token_ticker = hex::encode(token_ticker);
    let encoded_key = &hex::encode(SOV_TO_MVX_TOKEN_STORAGE_KEY);

    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .current_mvx_esdt_safe_contract_address()
                .clone()
                .to_address(),
            encoded_key,
            Some(&encoded_token_ticker),
        )
        .await;
}

/// ### TEST
/// M-ESDT_REG_OK
///
/// ### ACTION
/// Call 'register_token()' with valid token id and non-fungible type
///
/// ### EXPECTED
/// The token is registered
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn register_token_non_fungible_token() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    chain_interactor
        .deploy_chain_config(OptionalValue::None)
        .await;

    let contracts_array =
        chain_interactor.get_contract_info_struct_for_sc_type(vec![ScArray::ChainConfig]);

    chain_interactor
        .deploy_header_verifier(contracts_array)
        .await;

    chain_interactor
        .deploy_mvx_esdt_safe(OptionalValue::Some(EsdtSafeConfig::default_config()))
        .await;

    let sov_token_id = TokenIdentifier::from_esdt_bytes(SOV_TOKEN.as_str());
    let token_type = EsdtTokenType::NonFungible;
    let token_display_name = "SOVEREIGN";
    let num_decimals = 18;
    let token_ticker = TOKEN_TICKER;
    let egld_payment = BigUint::from(ISSUE_COST);

    chain_interactor
        .register_token(
            RegisterTokenArgs {
                sov_token_id,
                token_type,
                token_display_name,
                token_ticker,
                num_decimals,
            },
            egld_payment,
            None,
        )
        .await;

    let encoded_token_ticker = hex::encode(token_ticker);
    let encoded_key = &hex::encode(SOV_TO_MVX_TOKEN_STORAGE_KEY);

    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .current_mvx_esdt_safe_contract_address()
                .clone()
                .to_address(),
            encoded_key,
            Some(&encoded_token_ticker),
        )
        .await;
}

/// ### TEST
/// M-ESDT_REG_OK
///
/// ### ACTION
/// Call 'register_token()' with valid token id and dynamic NFT type
///
/// ### EXPECTED
/// The token is registered
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn register_token_dynamic_non_fungible_token() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    chain_interactor
        .deploy_chain_config(OptionalValue::None)
        .await;

    let contracts_array =
        chain_interactor.get_contract_info_struct_for_sc_type(vec![ScArray::ChainConfig]);

    chain_interactor
        .deploy_header_verifier(contracts_array)
        .await;

    chain_interactor
        .deploy_mvx_esdt_safe(OptionalValue::Some(EsdtSafeConfig::default_config()))
        .await;

    let sov_token_id = TokenIdentifier::from_esdt_bytes(SOV_TOKEN.as_str());
    let token_type = EsdtTokenType::DynamicNFT;
    let token_display_name = "SOVEREIGN";
    let num_decimals = 18;
    let token_ticker = TOKEN_TICKER;
    let egld_payment = BigUint::from(ISSUE_COST);

    chain_interactor
        .register_token(
            RegisterTokenArgs {
                sov_token_id,
                token_type,
                token_display_name,
                token_ticker,
                num_decimals,
            },
            egld_payment,
            None,
        )
        .await;

    let encoded_token_ticker = hex::encode(token_ticker);
    let encoded_key = &hex::encode(SOV_TO_MVX_TOKEN_STORAGE_KEY);

    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .current_mvx_esdt_safe_contract_address()
                .clone()
                .to_address(),
            encoded_key,
            Some(&encoded_token_ticker),
        )
        .await;
}

/// ### TEST
/// M-ESDT_EXEC_FAIL
///
/// ### ACTION
/// Call 'execute_operation()' with no esdt-safe-address set
///
/// ### EXPECTED
/// Error NO_ESDT_SAFE_ADDRESS
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn execute_operation_no_esdt_safe_registered() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    chain_interactor
        .deploy_chain_config(OptionalValue::None)
        .await;

    let contracts_array =
        chain_interactor.get_contract_info_struct_for_sc_type(vec![ScArray::ChainConfig]);

    chain_interactor
        .deploy_header_verifier(contracts_array)
        .await;

    chain_interactor
        .deploy_mvx_esdt_safe(OptionalValue::Some(EsdtSafeConfig::default_config()))
        .await;

    chain_interactor.unpause_endpoint().await;

    chain_interactor.deploy_testing_sc().await;

    let payment = OperationEsdtPayment::new(
        chain_interactor.state.get_first_token_id(),
        0,
        EsdtTokenData::default(),
    );

    let operation_data = OperationData::new(
        1,
        ManagedAddress::from_address(&chain_interactor.owner_address),
        None,
    );

    let operation = Operation::new(
        ManagedAddress::from_address(
            &chain_interactor
                .state
                .current_testing_sc_address()
                .to_address(),
        ),
        vec![payment].into(),
        operation_data,
    );

    let hash_of_hashes = chain_interactor.get_operation_hash(&operation);

    chain_interactor
        .execute_operations(
            hash_of_hashes,
            operation,
            Some(SETUP_PHASE_NOT_COMPLETED),
            None,
        )
        .await;

    let encoded_key = &hex::encode(OPERATION_HASH_STATUS_STORAGE_KEY);
    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .current_header_verifier_address()
                .to_address(),
            encoded_key,
            None,
        )
        .await;

    chain_interactor.check_wallet_balance().await;

    chain_interactor.check_testing_sc_balance_is_empty().await;

    chain_interactor.check_fee_market_balance_is_empty().await;
}

/// ### TEST
/// M-ESDT_EXEC_OK
///
/// ### ACTION
/// Call 'execute_operation()' with valid operation
///
/// ### EXPECTED
/// The operation is executed in the testing smart contract
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn execute_operation_success_no_fee() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let wallet_address = chain_interactor.wallet_address().clone();
    let token_data = EsdtTokenData {
        amount: BigUint::from(TEN_TOKENS),
        ..Default::default()
    };

    let payment =
        OperationEsdtPayment::new(chain_interactor.state.get_first_token_id(), 0, token_data);
    let mut payment_vec = PaymentsVec::new();
    payment_vec.push(EsdtTokenPayment {
        token_identifier: chain_interactor.state.get_first_token_id(),
        token_nonce: 0,
        amount: BigUint::from(TEN_TOKENS),
    });

    let gas_limit = 90_000_000u64;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = TransferData::new(gas_limit, function, args);

    let operation_data = OperationData::new(
        1,
        ManagedAddress::from_address(&chain_interactor.owner_address),
        Some(transfer_data),
    );

    chain_interactor
        .deploy_contracts(
            OptionalValue::None,
            OptionalValue::Some(EsdtSafeConfig::default_config()),
            None,
            vec![ScArray::ChainConfig, ScArray::ESDTSafe],
        )
        .await;

    chain_interactor.deploy_testing_sc().await;

    let operation = Operation::new(
        ManagedAddress::from_address(
            &chain_interactor
                .state
                .current_testing_sc_address()
                .to_address(),
        ),
        vec![payment].into(),
        operation_data,
    );

    let operation_hash = chain_interactor.get_operation_hash(&operation);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

    chain_interactor
        .deposit(
            chain_interactor
                .state
                .current_mvx_esdt_safe_contract_address()
                .to_address(),
            OptionalValue::None,
            payment_vec,
            None,
            Some("deposit"),
        )
        .await;

    let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));

    chain_interactor
        .register_operation(ManagedBuffer::new(), &hash_of_hashes, operations_hashes)
        .await;

    let operation_status = OperationHashStatus::NotLocked as u8;
    let expected_operation_hash_status = format!("{:02x}", operation_status);
    let encoded_key = &hex::encode(OPERATION_HASH_STATUS_STORAGE_KEY);

    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .current_header_verifier_address()
                .to_address(),
            encoded_key,
            Some(&expected_operation_hash_status),
        )
        .await;

    chain_interactor
        .execute_operations(hash_of_hashes, operation, None, Some("executedBridgeOp"))
        .await;

    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .current_header_verifier_address()
                .to_address(),
            encoded_key,
            None,
        )
        .await;

    let expected_tokens_wallet = vec![
        (
            chain_interactor.state.get_first_token_id().to_string(),
            BigUint::from(ONE_THOUSAND_TOKENS - TEN_TOKENS),
        ),
        chain_interactor.thousand_tokens(chain_interactor.state.get_second_token_id_string()),
        chain_interactor.thousand_tokens(chain_interactor.state.get_fee_token_id_string()),
    ];
    chain_interactor
        .check_address_balance(&Bech32Address::from(wallet_address), expected_tokens_wallet)
        .await;

    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty()
        .await;
    chain_interactor.check_fee_market_balance_is_empty().await;
}

/// ### TEST
/// M-ESDT_EXEC_OK
///
/// ### ACTION
/// Call 'execute_operation()' with valid operation and no fee
///
/// ### EXPECTED
/// The operation is executed in the testing smart contract
#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn execute_operation_only_transfer_data_no_fee() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;

    let gas_limit = 90_000_000u64;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = TransferData::new(gas_limit, function, args);

    let operation_data = OperationData::new(
        1,
        ManagedAddress::from_address(&chain_interactor.owner_address),
        Some(transfer_data),
    );

    chain_interactor
        .deploy_contracts(
            OptionalValue::None,
            OptionalValue::Some(EsdtSafeConfig::default_config()),
            None,
            vec![ScArray::ChainConfig, ScArray::ESDTSafe],
        )
        .await;

    chain_interactor.deploy_testing_sc().await;

    let operation = Operation::new(
        ManagedAddress::from_address(
            &chain_interactor
                .state
                .current_testing_sc_address()
                .to_address(),
        ),
        ManagedVec::new(),
        operation_data,
    );

    let operation_hash = chain_interactor.get_operation_hash(&operation);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

    let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));

    chain_interactor
        .register_operation(ManagedBuffer::new(), &hash_of_hashes, operations_hashes)
        .await;

    let operation_status = OperationHashStatus::NotLocked as u8;
    let expected_operation_hash_status = format!("{:02x}", operation_status);
    let encoded_key = &hex::encode(OPERATION_HASH_STATUS_STORAGE_KEY);

    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .current_header_verifier_address()
                .to_address(),
            encoded_key,
            Some(&expected_operation_hash_status),
        )
        .await;

    chain_interactor
        .execute_operations(hash_of_hashes, operation, None, Some("executedBridgeOp"))
        .await;

    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .current_header_verifier_address()
                .to_address(),
            encoded_key,
            None,
        )
        .await;

    chain_interactor.check_wallet_balance().await;
    chain_interactor
        .check_mvx_esdt_safe_balance_is_empty()
        .await;
    chain_interactor.check_fee_market_balance_is_empty().await;
}
