#![allow(non_snake_case)]

mod proxy;

use multiversx_sc_snippets::imports::*;
use multiversx_sc_snippets::sdk;
use serde::{Deserialize, Serialize};
use transaction::OperationData;
use transaction::PaymentsVec;
use transaction::{GasLimit, Operation};
use std::{
    io::{Read, Write},
    path::Path,
};


const GATEWAY: &str = sdk::gateway::DEVNET_GATEWAY;
const STATE_FILE: &str = "state.toml";
const TOKEN_ID: &[u8] = b"SVT-805b28";

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
        "deposit" => interact.deposit(OptionalTransferData::None).await,
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
        "addTokensToWhitelist" => interact.add_tokens_to_whitelist().await,
        "removeTokensFromWhitelist" => interact.remove_tokens_from_whitelist().await,
        "addTokensToBlacklist" => interact.add_tokens_to_blacklist().await,
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
    contract_address: Option<Bech32Address>
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
    bob_adress: Address,
    contract_code: BytesValue,
    state: State
}

impl ContractInteract {
    async fn new() -> Self {
        let mut interactor = Interactor::new(GATEWAY).await;
        let wallet_address = interactor.register_wallet(test_wallets::frank());
        let bob_adress = interactor.register_wallet(test_wallets::bob());
        
        let contract_code = BytesValue::interpret_from(
            "mxsc:../output/esdt-safe.mxsc.json",
            &InterpreterContext::default(),
        );

        ContractInteract {
            interactor,
            wallet_address,
            bob_adress,
            contract_code,
            state: State::load_state()
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
        self.state
            .set_address(Bech32Address::from_bech32_string(new_address_bech32.clone()));

        println!("new address: {new_address_bech32}");
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
        let fee_market_address = bech32::decode("");

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
        let header_verifier_address = bech32::decode("");

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

    async fn deposit(&mut self, transfer_data: OptionalTransferData<StaticApi>) {
        let token_id = TOKEN_ID;
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(20u64);

        let to = &self.bob_adress;
        let mut payments = PaymentsVec::new();
        payments.push(EsdtTokenPayment::new(TokenIdentifier::from(token_id), token_nonce, token_amount));
        payments.push(EsdtTokenPayment::new(TokenIdentifier::from(token_id), token_nonce, BigUint::from(30u64)));

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::EsdtSafeProxy)
            .deposit(to, transfer_data)
            .payment(payments)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
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
        let egld_amount = BigUint::<StaticApi>::from(0u128);

        let sov_token_id = TokenIdentifier::from_esdt_bytes(&b""[..]);
        let token_type = EsdtTokenType::NonFungible;
        let token_display_name = ManagedBuffer::new_from_bytes(&b""[..]);
        let token_ticker = ManagedBuffer::new_from_bytes(&b""[..]);
        let num_decimals = 0u32;

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::EsdtSafeProxy)
            .register_token(sov_token_id, token_type, token_display_name, token_ticker, num_decimals)
            .egld(egld_amount)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn execute_operations(&mut self) {
        let hash_of_hashes = ManagedBuffer::new_from_bytes(&b""[..]);
        let operation = Operation::new(ManagedAddress::zero(), ManagedVec::new(), OperationData::new(0 , ManagedAddress::zero(), None));

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(proxy::EsdtSafeProxy)
            .execute_operations(hash_of_hashes, operation)
            .returns(ReturnsResultUnmanaged)
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
        let _  = self
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
        let _  = self
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

        self
            .interactor
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

    async fn add_tokens_to_whitelist(&mut self) {
        let tokens = MultiValueVec::from(vec![TokenIdentifier::from_esdt_bytes(&b""[..])]);

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

    async fn add_tokens_to_blacklist(&mut self) {
        let tokens = MultiValueVec::from(vec![TokenIdentifier::from_esdt_bytes(&b""[..])]);

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
}

#[tokio::test]
async fn test_deploy() {
    let mut interact = ContractInteract::new().await;
    interact.deploy(false).await;
}
