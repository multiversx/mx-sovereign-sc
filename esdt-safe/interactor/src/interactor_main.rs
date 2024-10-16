#![allow(non_snake_case)]

mod proxies;

use fee_market::fee_market_proxy::FeeMarketProxy;
use fee_market::fee_market_proxy::{self, FeeStruct, FeeType};
use multiversx_sc_scenario::multiversx_chain_vm::crypto_functions::{
    self, sha256, SHA256_RESULT_LEN,
};
use multiversx_sc_snippets::imports::*;
use multiversx_sc_snippets::sdk::{self, crypto};
use proxies::*;
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
};
use transaction::OperationEsdtPayment;
use transaction::{GasLimit, Operation, OperationData, PaymentsVec};
const GATEWAY: &str = sdk::gateway::DEVNET_GATEWAY;
const STATE_FILE: &str = "state.toml";
const TOKEN_ID: &[u8] = b"SVT-805b28";
const WHITELISTED_TOKEN_ID: &[u8] = b"CHOCOLATE-daf625";

type OptionalTransferData<M> =
    OptionalValue<MultiValue3<GasLimit, ManagedBuffer<M>, ManagedVec<M, ManagedBuffer<M>>>>;

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut args = std::env::args();
    let _ = args.next();
    let cmd = args.next().expect("at least one argument required");
    let mut interact = ContractInteract::new().await;
    match cmd.as_str() {
        "deploy" => interact.deploy(false).await,
        "upgrade" => interact.upgrade().await,
        "setFeeMarketAddress" => interact.set_fee_market_address().await,
        "setHeaderVerifierAddress" => interact.set_header_verifier_address().await,
        "deposit" => interact.deposit(OptionalTransferData::None, None).await,
        "setMinValidSigners" => interact.set_min_valid_signers().await,
        "addSigners" => interact.add_signers().await,
        "removeSigners" => interact.remove_signers().await,
        "registerToken" => interact.register_token().await,
        "executeBridgeOps" => interact.execute_operations().await,
        "setMaxTxBatchSize" => interact.set_max_tx_batch_size().await,
        "setMaxTxBatchBlockDuration" => interact.set_max_tx_batch_block_duration().await,
        "getCurrentTxBatch" => interact.get_current_tx_batch().await,
        "getFirstBatchAnyStatus" => interact.get_first_batch_any_status().await,
        "getBatch" => interact.get_batch().await,
        "getBatchStatus" => interact.get_batch_status().await,
        "getFirstBatchId" => interact.first_batch_id().await,
        "getLastBatchId" => interact.last_batch_id().await,
        "setMaxBridgedAmount" => interact.set_max_bridged_amount().await,
        "getMaxBridgedAmount" => interact.max_bridged_amount().await,
        "endSetupPhase" => interact.end_setup_phase().await,
        "addTokensToWhitelist" => interact.add_tokens_to_whitelist(b"").await,
        "removeTokensFromWhitelist" => interact.remove_tokens_from_whitelist().await,
        "addTokensToBlacklist" => interact.add_tokens_to_blacklist(b"").await,
        "removeTokensFromBlacklist" => interact.remove_tokens_from_blacklist().await,
        "getTokenWhitelist" => interact.token_whitelist().await,
        "getTokenBlacklist" => interact.token_blacklist().await,
        "pause" => interact.pause_endpoint().await,
        "unpause" => interact.unpause_endpoint().await,
        "isPaused" => interact.paused_status().await,
        _ => panic!("unknown command: {}", &cmd),
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct State {
    contract_address: Option<Bech32Address>,
    fee_market_address: Option<Bech32Address>,
    price_aggregator_address: Option<Bech32Address>,
    header_verifier_address: Option<Bech32Address>,
}

impl State {
    // Deserializes state from file
    pub fn load_state() -> Self {
        if Path::new(STATE_FILE).exists() {
            let mut file = std::fs::File::open(STATE_FILE).unwrap();
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();
            toml::from_str(&content).unwrap()
        } else {
            Self::default()
        }
    }

    /// Sets the contract address
    pub fn set_address(&mut self, address: Bech32Address) {
        self.contract_address = Some(address);
    }

    pub fn set_fee_market_address(&mut self, address: Bech32Address) {
        self.fee_market_address = Some(address);
    }

    pub fn set_price_aggregator_address(&mut self, address: Bech32Address) {
        self.price_aggregator_address = Some(address);
    }

    pub fn set_header_verifier_address(&mut self, address: Bech32Address) {
        self.header_verifier_address = Some(address);
    }

    /// Returns the contract address
    pub fn current_address(&self) -> &Bech32Address {
        self.contract_address
            .as_ref()
            .expect("no known contract, deploy first")
    }
}

impl Drop for State {
    // Serializes state to file
    fn drop(&mut self) {
        let mut file = std::fs::File::create(STATE_FILE).unwrap();
        file.write_all(toml::to_string(self).unwrap().as_bytes())
            .unwrap();
    }
}

struct ContractInteract {
    interactor: Interactor,
    wallet_address: Address,
    bob_address: Address,
    alice_address: Address,
    mike_address: Address,
    judy_address: Address,
    contract_code: BytesValue,
    fee_market_code: BytesValue,
    price_aggregator_code: BytesValue,
    header_verifier_code: BytesValue,
    state: State,
}

impl ContractInteract {
    async fn new() -> Self {
        let mut interactor = Interactor::new(GATEWAY).await;
        let wallet_address = interactor.register_wallet(test_wallets::frank());
        let bob_address = interactor.register_wallet(test_wallets::bob());
        let alice_address = interactor.register_wallet(test_wallets::alice());
        let mike_address = interactor.register_wallet(test_wallets::mike());
        let judy_address = interactor.register_wallet(test_wallets::judy());

        let contract_code = BytesValue::interpret_from(
            "mxsc:../output/esdt-safe.mxsc.json",
            &InterpreterContext::default(),
        );

        let fee_market_code = BytesValue::interpret_from(
            "mxsc:contract-codes/fee-market.mxsc.json",
            &InterpreterContext::default(),
        );

        let price_aggregator_code = BytesValue::interpret_from(
            "mxsc:contract-codes/multiversx-price-aggregator-sc.mxsc.json",
            &InterpreterContext::default(),
        );

        let header_verifier_code = BytesValue::interpret_from(
            "mxsc:contract-codes/header-verifier.mxsc.json",
            &InterpreterContext::default(),
        );

        ContractInteract {
            interactor,
            wallet_address,
            bob_address,
            alice_address,
            mike_address,
            judy_address,
            contract_code,
            fee_market_code,
            price_aggregator_code,
            header_verifier_code,
            state: State::load_state(),
        }
    }

    async fn deploy(&mut self, is_sov_chain: bool) {
        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(110_000_000u64)
            .typed(proxy::EsdtSafeProxy)
            .init(is_sov_chain)
            .code(&self.contract_code)
            .returns(ReturnsNewAddress)
            .prepare_async()
            .run()
            .await;
        let new_address_bech32 = bech32::encode(&new_address);
        self.state.set_address(Bech32Address::from_bech32_string(
            new_address_bech32.clone(),
        ));

        println!("new address: {new_address_bech32}");
    }

    async fn deploy_fee_market(&mut self) {
        let fee = FeeStruct {
            base_token: TokenIdentifier::from_esdt_bytes(TOKEN_ID),
            fee_type: FeeType::Fixed {
                token: TokenIdentifier::from_esdt_bytes(TOKEN_ID),
                per_transfer: BigUint::from(10u64),
                per_gas: BigUint::from(0u64),
            },
        };

        let price_aggregator_address = managed_address!(self
            .state
            .price_aggregator_address
            .clone()
            .unwrap()
            .as_address());

        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(100_000_000u64)
            .typed(fee_market_proxy::FeeMarketProxy)
            .init(
                self.state.current_address(),
                price_aggregator_address,
                Option::Some(fee),
            )
            .code(&self.fee_market_code)
            .returns(ReturnsNewAddress)
            .prepare_async()
            .run()
            .await;
        let new_address_bech32 = bech32::encode(&new_address);
        self.state
            .set_fee_market_address(Bech32Address::from_bech32_string(
                new_address_bech32.clone(),
            ));
        println!("new fee_market_address: {new_address_bech32}");
    }

    async fn deploy_price_aggregator(&mut self) {
        let mut oracles = MultiValueEncoded::new();
        let first_oracle_adress = managed_address!(&self.bob_address.clone());
        let second_oracle_adress = managed_address!(&self.alice_address.clone());
        let third_oracle_adress = managed_address!(&self.mike_address.clone());
        let forth_oracle_address = managed_address!(&self.judy_address.clone());
        oracles.push(first_oracle_adress);
        oracles.push(second_oracle_adress);
        oracles.push(third_oracle_adress);
        oracles.push(forth_oracle_address);

        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(100_000_000u64)
            .typed(price_aggregator_proxy::PriceAggregatorProxy)
            .init(
                TokenIdentifier::from_esdt_bytes(TOKEN_ID),
                BigUint::from(1u64),
                BigUint::from(1u64),
                3u8,
                3u8,
                oracles,
            )
            .code(&self.price_aggregator_code)
            .returns(ReturnsNewAddress)
            .prepare_async()
            .run()
            .await;
        let new_address_bech32 = bech32::encode(&new_address);
        self.state
            .set_price_aggregator_address(Bech32Address::from_bech32_string(
                new_address_bech32.clone(),
            ));
        println!("new token_handler_address: {new_address_bech32}");
    }

    async fn deploy_header_verifier_contract(&mut self) {
        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(100_000_000u64)
            .typed(proxy::EsdtSafeProxy)
            .init(false)
            .code(&self.header_verifier_code)
            .returns(ReturnsNewAddress)
            .prepare_async()
            .run()
            .await;
        let new_address_bech32 = bech32::encode(&new_address);
        self.state
            .set_header_verifier_address(Bech32Address::from_bech32_string(
                new_address_bech32.clone(),
            ));

        println!("new header_verifier_address: {new_address_bech32}");
    }

    async fn upgrade(&mut self) {
        let response = self
            .interactor
            .tx()
            .to(self.state.current_address())
            .from(&self.wallet_address)
            .gas(30_000_000u64)
            .typed(proxy::EsdtSafeProxy)
            .upgrade()
            .code(&self.contract_code)
            .code_metadata(CodeMetadata::UPGRADEABLE)
            .returns(ReturnsNewAddress)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn set_fee_market_address(&mut self) {
        let fee_market_address = self.state.fee_market_address.clone().unwrap();
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::EsdtSafeProxy)
            .set_fee_market_address(fee_market_address)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn set_header_verifier_address(&mut self) {
        let header_verifier_address = self.state.header_verifier_address.clone().unwrap();

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::EsdtSafeProxy)
            .set_header_verifier_address(header_verifier_address)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn deposit(
        &mut self,
        transfer_data: OptionalTransferData<StaticApi>,
        expect_error: Option<ExpectError<'_>>,
    ) {
        let token_id = TOKEN_ID;
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(20u64);

        let to = &self.bob_address;
        let mut payments = PaymentsVec::new();
        payments.push(EsdtTokenPayment::new(
            TokenIdentifier::from(token_id),
            token_nonce,
            token_amount,
        ));

        match expect_error {
            Some(error) => {
                self.interactor
                    .tx()
                    .from(&self.wallet_address)
                    .to(self.state.current_address())
                    .gas(90_000_000u64)
                    .typed(proxy::EsdtSafeProxy)
                    .deposit(to, transfer_data)
                    .payment(payments)
                    .returns(error)
                    .prepare_async()
                    .run()
                    .await;
            }
            None => {
                self.interactor
                    .tx()
                    .from(&self.wallet_address)
                    .to(self.state.current_address())
                    .gas(90_000_000u64)
                    .typed(proxy::EsdtSafeProxy)
                    .deposit(to, transfer_data)
                    .payment(payments)
                    .returns(ReturnsResultUnmanaged)
                    .prepare_async()
                    .run()
                    .await;
            }
        }
    }

    async fn set_min_valid_signers(&mut self) {
        let new_value = 0u32;

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::EsdtSafeProxy)
            .set_min_valid_signers(new_value)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn add_signers(&mut self) {
        let signers = MultiValueVec::from(vec![bech32::decode("")]);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::EsdtSafeProxy)
            .add_signers(signers)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn remove_signers(&mut self) {
        let signers = MultiValueVec::from(vec![bech32::decode("")]);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::EsdtSafeProxy)
            .remove_signers(signers)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn register_token(&mut self) {
        let egld_amount = BigUint::<StaticApi>::from(50_000_000_000_000_000u64);

        let sov_token_id = TokenIdentifier::from_esdt_bytes(&b"SOV"[..]);
        let token_type = EsdtTokenType::Fungible;
        let token_display_name = ManagedBuffer::new_from_bytes(&b"SOVEREIGN"[..]);
        let token_ticker = ManagedBuffer::new_from_bytes(&b"SVCT"[..]);
        let num_decimals = 18u32;

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(90_000_000u64)
            .typed(proxy::EsdtSafeProxy)
            .register_token(
                sov_token_id,
                token_type,
                token_display_name,
                token_ticker,
                num_decimals,
            )
            .egld(egld_amount)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn execute_operations(&mut self) {
        let (tokens, data) = self.setup_payments().await;
        let to = managed_address!(&self.bob_address);
        let operation = Operation::new(to, tokens, data);
        let operation_hash = self.get_operation_hash(&operation).await;

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::EsdtSafeProxy)
            .execute_operations(operation_hash, operation)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn execute_operations_with_error(&mut self, error_msg: ExpectError<'_>) {
        let (tokens, data) = self.setup_payments().await;
        let to = managed_address!(&self.bob_address);
        let operation = Operation::new(to, tokens, data);
        let operation_hash = self.get_operation_hash(&operation).await;

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::EsdtSafeProxy)
            .execute_operations(operation_hash, operation)
            .returns(error_msg)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn set_max_tx_batch_size(&mut self) {
        let new_max_tx_batch_size = 0u32;

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::EsdtSafeProxy)
            .set_max_tx_batch_size(new_max_tx_batch_size)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn set_max_tx_batch_block_duration(&mut self) {
        let new_max_tx_batch_block_duration = 0u64;

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::EsdtSafeProxy)
            .set_max_tx_batch_block_duration(new_max_tx_batch_block_duration)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn get_current_tx_batch(&mut self) {
        let _ = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::EsdtSafeProxy)
            .get_current_tx_batch()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;
    }

    async fn get_first_batch_any_status(&mut self) {
        let _ = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::EsdtSafeProxy)
            .get_first_batch_any_status()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;
    }

    async fn get_batch(&mut self) {
        let batch_id = 0u64;

        let _ = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::EsdtSafeProxy)
            .get_batch(batch_id)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;
    }

    async fn get_batch_status(&mut self) {
        let batch_id = 0u64;

        self.interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::EsdtSafeProxy)
            .get_batch_status(batch_id)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;
    }

    async fn first_batch_id(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::EsdtSafeProxy)
            .first_batch_id()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn last_batch_id(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::EsdtSafeProxy)
            .last_batch_id()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn set_max_bridged_amount(&mut self) {
        let token_id = TokenIdentifier::from_esdt_bytes(&b""[..]);
        let max_amount = BigUint::<StaticApi>::from(0u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::EsdtSafeProxy)
            .set_max_bridged_amount(token_id, max_amount)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn max_bridged_amount(&mut self) {
        let token_id = TokenIdentifier::from_esdt_bytes(&b""[..]);

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::EsdtSafeProxy)
            .max_bridged_amount(token_id)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn end_setup_phase(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::EsdtSafeProxy)
            .end_setup_phase()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn add_tokens_to_whitelist(&mut self, token_id: &[u8]) {
        let tokens = MultiValueVec::from(vec![TokenIdentifier::from_esdt_bytes(token_id)]);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::EsdtSafeProxy)
            .add_tokens_to_whitelist(tokens)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn remove_tokens_from_whitelist(&mut self) {
        let tokens = MultiValueVec::from(vec![TokenIdentifier::from_esdt_bytes(&b""[..])]);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::EsdtSafeProxy)
            .remove_tokens_from_whitelist(tokens)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn add_tokens_to_blacklist(&mut self, token_id: &[u8]) {
        let tokens = MultiValueVec::from(vec![TokenIdentifier::from_esdt_bytes(token_id)]);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::EsdtSafeProxy)
            .add_tokens_to_blacklist(tokens)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn remove_tokens_from_blacklist(&mut self) {
        let tokens = MultiValueVec::from(vec![TokenIdentifier::from_esdt_bytes(&b""[..])]);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::EsdtSafeProxy)
            .remove_tokens_from_blacklist(tokens)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn token_whitelist(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::EsdtSafeProxy)
            .token_whitelist()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn token_blacklist(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::EsdtSafeProxy)
            .token_blacklist()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn pause_endpoint(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::EsdtSafeProxy)
            .pause_endpoint()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn unpause_endpoint(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::EsdtSafeProxy)
            .unpause_endpoint()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn paused_status(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::EsdtSafeProxy)
            .paused_status()
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn disable_fee(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.fee_market_address.clone().unwrap().as_address())
            .gas(30_000_000u64)
            .typed(FeeMarketProxy)
            .disable_fee(TOKEN_ID)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn register_operations(&mut self, operation: &Operation<StaticApi>) {
        let bls_signature = ManagedByteArray::default();
        let operation_hash = self.get_operation_hash(operation);
        let hash_of_hashes = sha256(&operation_hash);

        let mut managed_operation_hashes =
            MultiValueEncoded::<StaticApi, ManagedBuffer<StaticApi>>::new();

        let managed_operation_hash = ManagedBuffer::<StaticApi>::from(&operation_hash);
        let managed_hash_of_hashes = ManagedBuffer::<StaticApi>::from(&hash_of_hashes);

        managed_operation_hashes.push(managed_operation_hash);

        if let Some(header_verifier_address) = self.state.header_verifier_address.clone() {
            self.interactor
                .tx()
                .from(&self.wallet_address)
                .to(header_verifier_address)
                .typed(header_verifier_proxy::HeaderverifierProxy)
                .register_bridge_operations(
                    bls_signature,
                    managed_hash_of_hashes,
                    managed_operation_hashes,
                )
                .returns(ReturnsResult)
                .prepare_async()
                .run()
                .await;
        }
    }

    async fn setup_payments(
        &mut self,
    ) -> (
        ManagedVec<StaticApi, OperationEsdtPayment<StaticApi>>,
        OperationData<StaticApi>,
    ) {
        let mut tokens: ManagedVec<StaticApi, OperationEsdtPayment<StaticApi>> = ManagedVec::new();
        let token_ids = vec![TOKEN_ID];

        for token_id in token_ids {
            let payment: OperationEsdtPayment<StaticApi> = OperationEsdtPayment {
                token_identifier: token_id.into(),
                token_nonce: 1,
                token_data: EsdtTokenData::default(),
            };

            tokens.push(payment);
        }

        let op_sender = managed_address!(&self.wallet_address);
        let data: OperationData<StaticApi> = OperationData {
            op_nonce: 1,
            op_sender,
            opt_transfer_data: Option::None,
        };

        (tokens, data)
    }

    fn get_operation_hash(&mut self, operation: &Operation<StaticApi>) -> [u8; SHA256_RESULT_LEN] {
        let mut serialized_operation: ManagedBuffer<StaticApi> = ManagedBuffer::new();
        let _ = operation.top_encode(&mut serialized_operation);
        sha256(&serialized_operation.to_vec())
    }

    fn get_hash(&mut self, operation: &ManagedBuffer<StaticApi>) -> ManagedBuffer<StaticApi> {
        let mut array = [0; 1024];

        let len = {
            let byte_array = operation.load_to_byte_array(&mut array);
            byte_array.len()
        };

        let trimmed_slice = &array[..len];
        let hash = sha256(trimmed_slice);

        ManagedBuffer::from(&hash)
    }
}

#[tokio::test]
async fn test_deploy_sov() {
    let mut interact = ContractInteract::new().await;
    interact.deploy(true).await;
    interact.deploy_price_aggregator().await;
    interact.deploy_fee_market().await;
    interact.set_fee_market_address().await;
    interact.disable_fee().await;
    interact.deploy_header_verifier_contract().await;
    interact.set_header_verifier_address().await;
    interact.unpause_endpoint().await;
}

#[tokio::test]
async fn test_register_operation() {
    let mut interact = ContractInteract::new().await;
}
