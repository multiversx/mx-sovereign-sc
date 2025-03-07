use multiversx_sc_snippets::imports::*;
use multiversx_sc_snippets::multiversx_sc_scenario::multiversx_chain_vm::crypto_functions::sha256;
use proxies::fee_market_proxy::{FeeStruct, FeeType};
use rust_interact::config::Config;
use rust_interact::mvx_esdt_safe::mvx_esdt_safe_interactor_main::MvxEsdtSafeInteract;
use rust_interact::{
    RegisterTokenArgs, FEE_TOKEN, FIRST_TOKEN, ISSUE_COST, MVX_TO_SOV_TOKEN_STORAGE_KEY,
    SECOND_TOKEN, SOV_TO_MVX_TOKEN_STORAGE_KEY,
};
use serial_test::serial;
use structs::aliases::PaymentsVec;
use structs::configs::EsdtSafeConfig;
use structs::operation::{Operation, OperationData, OperationEsdtPayment, TransferData};

#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn deposit_nothing_to_transfer() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    chain_interactor.deploy_header_verifier().await;
    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();
    chain_interactor
        .deploy_mvx_esdt_safe(
            chain_interactor
                .state
                .current_header_verifier_address()
                .clone(),
            OptionalValue::Some(EsdtSafeConfig::default_config()),
        )
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor.unpause_endpoint().await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor
        .deposit(
            chain_interactor.bob_address.clone(),
            OptionalValue::None,
            ManagedVec::new(),
            Some("Nothing to transfer"),
        )
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();
}

#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn deposit_too_many_tokens() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    chain_interactor.deploy_header_verifier().await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();
    chain_interactor
        .deploy_mvx_esdt_safe(
            chain_interactor
                .state
                .current_header_verifier_address()
                .clone(),
            OptionalValue::Some(EsdtSafeConfig::default_config()),
        )
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor.unpause_endpoint().await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    let esdt_token_payment = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(FIRST_TOKEN),
        0,
        BigUint::from(1u64),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment; 11]);

    chain_interactor
        .deposit(
            chain_interactor.bob_address.clone(),
            OptionalValue::None,
            payments_vec,
            Some("Too many tokens"),
        )
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();
}

#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn deposit_no_transfer_data() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    chain_interactor.deploy_header_verifier().await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor
        .deploy_mvx_esdt_safe(
            chain_interactor
                .state
                .current_header_verifier_address()
                .clone(),
            OptionalValue::Some(EsdtSafeConfig::default_config()),
        )
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor.unpause_endpoint().await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor
        .deploy_fee_market(
            chain_interactor
                .state
                .current_mvx_esdt_safe_contract_address()
                .clone(),
            None,
        )
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor
        .set_fee_market_address(
            chain_interactor
                .state
                .current_fee_market_address()
                .clone()
                .to_address(),
        )
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(FIRST_TOKEN),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(SECOND_TOKEN),
        0,
        BigUint::from(100u64),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one, esdt_token_payment_two]);

    chain_interactor
        .deposit(
            chain_interactor.bob_address.clone(),
            OptionalValue::None,
            payments_vec,
            None,
        )
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .current_mvx_esdt_safe_contract_address()
                .clone()
                .to_address(),
            MVX_TO_SOV_TOKEN_STORAGE_KEY,
            None,
        )
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();
}

#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn deposit_gas_limit_too_high() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let config = EsdtSafeConfig::new(ManagedVec::new(), ManagedVec::new(), 1, ManagedVec::new());
    chain_interactor.deploy_header_verifier().await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor
        .deploy_mvx_esdt_safe(
            chain_interactor
                .state
                .current_header_verifier_address()
                .clone(),
            OptionalValue::Some(config),
        )
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor.unpause_endpoint().await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor
        .deploy_fee_market(
            chain_interactor
                .state
                .current_mvx_esdt_safe_contract_address()
                .clone(),
            None,
        )
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor
        .set_fee_market_address(
            chain_interactor
                .state
                .current_fee_market_address()
                .clone()
                .to_address(),
        )
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor.deploy_testing_sc().await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(FIRST_TOKEN),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(SECOND_TOKEN),
        0,
        BigUint::from(100u64),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one, esdt_token_payment_two]);

    let gas_limit = 2;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    chain_interactor
        .deposit(
            chain_interactor.bob_address.clone(),
            OptionalValue::Some(transfer_data),
            payments_vec,
            Some("Gas limit too high"),
        )
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();
}

#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn deposit_endpoint_banned() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::from(vec![ManagedBuffer::from("hello")]),
    );
    chain_interactor.deploy_header_verifier().await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor
        .deploy_mvx_esdt_safe(
            chain_interactor
                .state
                .current_header_verifier_address()
                .clone(),
            OptionalValue::Some(config),
        )
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor.unpause_endpoint().await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor
        .deploy_fee_market(
            chain_interactor
                .state
                .current_mvx_esdt_safe_contract_address()
                .clone(),
            None,
        )
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor
        .set_fee_market_address(
            chain_interactor
                .state
                .current_fee_market_address()
                .clone()
                .to_address(),
        )
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor.deploy_testing_sc().await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(FIRST_TOKEN),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(SECOND_TOKEN),
        0,
        BigUint::from(100u64),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one, esdt_token_payment_two]);

    let gas_limit = 2;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    chain_interactor
        .deposit(
            chain_interactor.bob_address.clone(),
            OptionalValue::Some(transfer_data),
            payments_vec,
            Some("Banned endpoint name"),
        )
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();
}

// NOTE: Add checks for account storage after finding out how to encode values in state
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
    );

    let per_transfer = BigUint::from(1u64);
    let per_gas = BigUint::from(1u64);

    let fee = FeeStruct {
        base_token: TokenIdentifier::from(FEE_TOKEN),
        fee_type: FeeType::Fixed {
            token: TokenIdentifier::from(FEE_TOKEN),
            per_transfer: per_transfer.clone(),
            per_gas: per_gas.clone(),
        },
    };

    chain_interactor.deploy_header_verifier().await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor
        .deploy_mvx_esdt_safe(
            chain_interactor
                .state
                .current_header_verifier_address()
                .clone(),
            OptionalValue::Some(config),
        )
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor.unpause_endpoint().await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor
        .deploy_fee_market(
            chain_interactor
                .state
                .current_mvx_esdt_safe_contract_address()
                .clone(),
            Some(fee),
        )
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor
        .set_fee_market_address(
            chain_interactor
                .state
                .current_fee_market_address()
                .clone()
                .to_address(),
        )
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor.deploy_testing_sc().await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    let fee_amount = BigUint::from(10_000u64);

    let fee_payment =
        EsdtTokenPayment::<StaticApi>::new(TokenIdentifier::from(FEE_TOKEN), 0, fee_amount.clone());

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(FIRST_TOKEN),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(SECOND_TOKEN),
        0,
        BigUint::from(100u64),
    );

    let payments_vec = PaymentsVec::from(vec![
        fee_payment,
        esdt_token_payment_one.clone(),
        esdt_token_payment_two.clone(),
    ]);

    let gas_limit = 1000u64;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    chain_interactor
        .deposit(
            chain_interactor.bob_address.clone(),
            OptionalValue::Some(transfer_data),
            payments_vec,
            None,
        )
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();
}

#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn deposit_payment_doesnt_cover_fee() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::new(),
    );

    let per_transfer = BigUint::from(1u64);
    let per_gas = BigUint::from(1u64);

    let fee = FeeStruct {
        base_token: TokenIdentifier::from(FIRST_TOKEN),
        fee_type: FeeType::Fixed {
            token: TokenIdentifier::from(FIRST_TOKEN),
            per_transfer: per_transfer.clone(),
            per_gas: per_gas.clone(),
        },
    };

    chain_interactor.deploy_header_verifier().await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor
        .deploy_mvx_esdt_safe(
            chain_interactor
                .state
                .current_header_verifier_address()
                .clone(),
            OptionalValue::Some(config),
        )
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor
        .deploy_fee_market(
            chain_interactor
                .state
                .current_mvx_esdt_safe_contract_address()
                .clone(),
            Some(fee.clone()),
        )
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor
        .set_fee_market_address(
            chain_interactor
                .state
                .current_fee_market_address()
                .clone()
                .to_address(),
        )
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor.deploy_testing_sc().await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor.unpause_endpoint().await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(FIRST_TOKEN),
        0,
        BigUint::from(1u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(SECOND_TOKEN),
        0,
        BigUint::from(100u64),
    );

    let payments_vec = PaymentsVec::from(vec![
        esdt_token_payment_one.clone(),
        esdt_token_payment_two.clone(),
    ]);

    let gas_limit = 10_000;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    chain_interactor
        .deposit(
            chain_interactor.bob_address.clone(),
            OptionalValue::Some(transfer_data),
            payments_vec,
            Some("Payment does not cover fee"),
        )
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();
}

// TODO: add deposit_refund_fee test after finding a method to check for balance

#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn register_token_invalid_type() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    chain_interactor.deploy_header_verifier().await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor
        .deploy_mvx_esdt_safe(
            chain_interactor
                .state
                .current_header_verifier_address()
                .clone(),
            OptionalValue::Some(EsdtSafeConfig::default_config()),
        )
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    let sov_token_id = TokenIdentifier::from_esdt_bytes(FIRST_TOKEN);
    let token_type = EsdtTokenType::Invalid;
    let token_display_name = "SOVEREIGN";
    let num_decimals = 2;
    let token_ticker = FIRST_TOKEN;
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
            Some("Invalid type"),
        )
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();
}

#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn register_token_fungible_token() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    chain_interactor.deploy_header_verifier().await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor
        .deploy_mvx_esdt_safe(
            chain_interactor
                .state
                .current_header_verifier_address()
                .clone(),
            OptionalValue::Some(EsdtSafeConfig::default_config()),
        )
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    let sov_token_id = TokenIdentifier::from_esdt_bytes(FIRST_TOKEN);
    let token_type = EsdtTokenType::Fungible;
    let token_display_name = "GREEN";
    let num_decimals = 2;
    let token_ticker = "GREEN";
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

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .current_mvx_esdt_safe_contract_address()
                .clone()
                .to_address(),
            SOV_TO_MVX_TOKEN_STORAGE_KEY,
            Some(token_ticker),
        )
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();
}

#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn register_token_non_fungible() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    chain_interactor.deploy_header_verifier().await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor
        .deploy_mvx_esdt_safe(
            chain_interactor
                .state
                .current_header_verifier_address()
                .clone(),
            OptionalValue::Some(EsdtSafeConfig::default_config()),
        )
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    let sov_token_id = TokenIdentifier::from_esdt_bytes(FIRST_TOKEN);
    let token_type = EsdtTokenType::NonFungible;
    let token_display_name = "GREEN";
    let num_decimals = 2;
    let token_ticker = "GREEN";
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

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .current_mvx_esdt_safe_contract_address()
                .clone()
                .to_address(),
            SOV_TO_MVX_TOKEN_STORAGE_KEY,
            Some(token_ticker),
        )
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();
}

#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn register_token_dynamic_non_fungible() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    chain_interactor.deploy_header_verifier().await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor
        .deploy_mvx_esdt_safe(
            chain_interactor
                .state
                .current_header_verifier_address()
                .clone(),
            OptionalValue::Some(EsdtSafeConfig::default_config()),
        )
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    let sov_token_id = TokenIdentifier::from_esdt_bytes(FIRST_TOKEN);
    let token_type = EsdtTokenType::DynamicNFT;
    let token_display_name = "GREEN";
    let num_decimals = 2;
    let token_ticker = "GREEN";
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

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor
        .check_account_storage(
            chain_interactor
                .state
                .current_mvx_esdt_safe_contract_address()
                .clone()
                .to_address(),
            SOV_TO_MVX_TOKEN_STORAGE_KEY,
            Some(token_ticker),
        )
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();
}

#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn execute_operation_no_esdt_safe_registered() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    chain_interactor.deploy_header_verifier().await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor
        .deploy_mvx_esdt_safe(
            chain_interactor
                .state
                .current_header_verifier_address()
                .clone(),
            OptionalValue::Some(EsdtSafeConfig::default_config()),
        )
        .await;

    chain_interactor.unpause_endpoint().await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    let payment = OperationEsdtPayment::new(
        TokenIdentifier::from(FIRST_TOKEN),
        0,
        EsdtTokenData::default(),
    );

    let operation_data = OperationData::new(
        1,
        ManagedAddress::from_address(&chain_interactor.wallet_address),
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
            operation.clone(),
            Some("There is no registered ESDT address"),
        )
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();
}

#[tokio::test]
#[serial]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn execute_operation_success() {
    let mut chain_interactor = MvxEsdtSafeInteract::new(Config::chain_simulator_config()).await;
    let config = OptionalValue::Some(EsdtSafeConfig::default_config());
    let token_data = EsdtTokenData {
        amount: BigUint::from(100u64),
        ..Default::default()
    };

    let payment = OperationEsdtPayment::new(TokenIdentifier::from(FIRST_TOKEN), 0, token_data);

    let gas_limit = 90_000_000u64;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = TransferData::new(gas_limit, function, args);

    let operation_data = OperationData::new(
        1,
        ManagedAddress::from_address(&chain_interactor.wallet_address),
        Some(transfer_data),
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

    let operation_hash = chain_interactor.get_operation_hash(&operation);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

    chain_interactor.deploy_header_verifier().await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor.deploy_testing_sc().await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor
        .deploy_mvx_esdt_safe(
            chain_interactor
                .state
                .current_header_verifier_address()
                .clone(),
            config,
        )
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor.unpause_endpoint().await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor
        .set_esdt_safe_address_in_header_verifier()
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    let sov_token_id = TokenIdentifier::from_esdt_bytes(FIRST_TOKEN);
    let token_type = EsdtTokenType::Fungible;
    let token_display_name = "GREEN";
    let num_decimals = 2;
    let token_ticker = "GREEN";
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

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));

    chain_interactor.deploy_chain_config().await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor
        .register_operation(ManagedBuffer::new(), &hash_of_hashes, operations_hashes)
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();

    chain_interactor
        .execute_operations(hash_of_hashes, operation.clone(), None)
        .await;

    chain_interactor
        .interactor
        .generate_blocks(2u64)
        .await
        .unwrap();
}
