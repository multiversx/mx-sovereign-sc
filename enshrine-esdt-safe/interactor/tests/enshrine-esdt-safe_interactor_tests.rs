use std::vec;

use aliases::{GasLimit, PaymentsVec};
use enshrine_esdt_safe_interactor::ContractInteract;
use interactor::constants::{TOKEN_ID, WHITELIST_TOKEN_ID};
use interactor::interactor_config::Config;
use multiversx_sc_snippets::imports::*;
use operation::*;
use proxies::*;

type OptionalTransferData<M> =
    OptionalValue<MultiValue3<GasLimit, ManagedBuffer<M>, ManagedVec<M, ManagedBuffer<M>>>>;

#[tokio::test]
#[ignore]
async fn test_deposit_paused() {
    let mut interact = ContractInteract::new(Config::load_config()).await;
    interact.deploy_token_handler().await;
    interact.deploy(false, BridgeConfig::default_config()).await;
    interact
        .deposit(
            OptionalTransferData::None,
            Some(ExpectError(4, "Cannot create transaction while paused")),
        )
        .await;
}

#[tokio::test]
#[ignore]
async fn test_deposit_no_payment() {
    let mut interact = ContractInteract::new(Config::load_config()).await;
    let to = interact.bob_address.clone();
    let from = interact.wallet_address.clone();
    let to_contract = interact.state.esdt_safe_address().clone();
    let transfer_data = OptionalTransferData::None;

    interact.deploy_setup(BridgeConfig::default_config()).await;

    interact
        .interactor
        .tx()
        .from(from)
        .to(to_contract)
        .gas(30_000_000u64)
        .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
        .deposit(to, transfer_data)
        .returns(ExpectError(4, "Nothing to transfer"))
        .run()
        .await;
}

#[tokio::test]
#[ignore]
async fn test_deposit_too_many_payments() {
    let mut interact = ContractInteract::new(Config::load_config()).await;
    let to = interact.bob_address.clone();
    let from = interact.wallet_address.clone();
    let to_contract = interact.state.esdt_safe_address().clone();
    let transfer_data = OptionalTransferData::None;
    let payment = EsdtTokenPayment::new(
        TokenIdentifier::from_esdt_bytes(TOKEN_ID),
        0u64,
        BigUint::from(10u64),
    );
    let payments = ManagedVec::from(vec![payment; 11]);

    interact.deploy_setup(BridgeConfig::default_config()).await;

    interact
        .interactor
        .tx()
        .from(from)
        .to(to_contract)
        .gas(30_000_000u64)
        .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
        .deposit(to, transfer_data)
        .payment(payments)
        .returns(ExpectError(4, "Too many tokens"))
        .run()
        .await;
}

#[tokio::test]
#[ignore]
async fn test_deposit_not_whitelisted() {
    let mut interact = ContractInteract::new(Config::load_config()).await;
    interact.deploy_setup(BridgeConfig::default_config()).await;
    interact.deploy_fee_market().await;
    interact.add_tokens_to_whitelist(WHITELIST_TOKEN_ID).await;
    interact.set_fee_market_address().await;
    interact.deposit(OptionalTransferData::None, None).await;
}

#[tokio::test]
#[ignore]
async fn test_deposit_happy_path() {
    let mut interact = ContractInteract::new(Config::load_config()).await;
    interact.deploy_setup(BridgeConfig::default_config()).await;
    interact.deploy_fee_market().await;
    interact.add_tokens_to_whitelist(TOKEN_ID).await;
    interact.set_fee_market_address().await;
    interact.deposit(OptionalTransferData::None, None).await;
}

// FAILS => Waiting for fixes (initiator address not set)
#[tokio::test]
#[ignore]
async fn test_deposit_sov_chain() {
    let mut interact = ContractInteract::new(Config::load_config()).await;
    let transfer_data = OptionalTransferData::None;
    let mut payments = PaymentsVec::new();
    payments.push(EsdtTokenPayment::new(
        TokenIdentifier::from(TOKEN_ID),
        0,
        BigUint::from(10u64),
    ));
    payments.push(EsdtTokenPayment::new(
        TokenIdentifier::from(TOKEN_ID),
        0,
        BigUint::from(30u64),
    ));
    interact
        .deploy_all(true, BridgeConfig::default_config())
        .await;
    interact.add_tokens_to_whitelist(TOKEN_ID).await;
    interact.set_fee_market_address().await;
    interact
        .interactor
        .tx()
        .from(interact.wallet_address)
        .to(interact.state.esdt_safe_address())
        .gas(30_000_000u64)
        .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
        .deposit(interact.state.esdt_safe_address(), transfer_data)
        .payment(payments)
        .returns(ReturnsResultUnmanaged)
        .run()
        .await;
}
