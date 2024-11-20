#![allow(non_snake_case)]
// TODO: Remove this when interactor setup is complete
#![allow(dead_code)]

use multiversx_sc_scenario::multiversx_chain_vm::crypto_functions::{sha256, SHA256_RESULT_LEN};
use multiversx_sc_scenario::scenario_model::TxResponseStatus;
use multiversx_sc_snippets::imports::*;
use multiversx_sc_snippets::sdk::{self};
use proxies::esdt_safe_proxy::EsdtSafeProxy;
use proxies::fee_market_proxy::{FeeMarketProxy, FeeStruct, FeeType};
use proxies::header_verifier_proxy::HeaderverifierProxy;
use proxies::testing_sc_proxy::TestingScProxy;
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
};
use transaction::{GasLimit, Operation, OperationData, PaymentsVec};
use transaction::{OperationEsdtPayment, TransferData};
const GATEWAY: &str = sdk::gateway::DEVNET_GATEWAY;
const STATE_FILE: &str = "state.toml";
const TOKEN_ID: &[u8] = b"SOV-101252";
const TOKEN_ID_FOR_EXECUTE: &[u8] = b"x-SOV-101252";
const WHITELISTED_TOKEN_ID: &[u8] = b"CHOCOLATE-daf625";
const INTERACTOR_SCENARIO_TRACE_PATH: &str = "interactor_trace.scen.json";
const FEE_MARKET_CODE_PATH: &str = "../../fee-market/output/fee-market.mxsc.json";
const HEADER_VERIFIER_CODE_PATH: &str = "../../header-verifier/output/header-verifier.mxsc.json";
const ESDT_SAFE_CODE_PATH: &str = "../output/esdt-safe.mxsc.json";
const TESTING_SC_CODE_PATH: &str = "../../testing-sc/output/testing-sc.mxsc.json";

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
        "registerToken" => interact.register_token().await,
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
    header_verifier_address: Option<Bech32Address>,
    testing_sc_address: Option<Bech32Address>,
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

    pub fn set_header_verifier_address(&mut self, address: Bech32Address) {
        self.header_verifier_address = Some(address);
    }

    pub fn set_testing_sc_address(&mut self, address: Bech32Address) {
        self.testing_sc_address = Some(address);
    }

    /// Returns the contract address
    pub fn current_address(&self) -> &Bech32Address {
        self.contract_address
            .as_ref()
            .expect("no known contract, deploy first")
    }

    pub fn get_header_verifier_address(&self) -> Address {
        self.header_verifier_address.clone().unwrap().to_address()
    }

    pub fn get_fee_market_address(&self) -> Address {
        self.fee_market_address.clone().unwrap().to_address()
    }

    pub fn get_testing_sc_address(&self) -> Address {
        self.testing_sc_address.clone().unwrap().to_address()
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
    frank_address: Address,
    alice_address: Address,
    mike_address: Address,
    judy_address: Address,
    esdt_safe_code: String,
    fee_market_code: String,
    header_verifier_code: String,
    testing_sc_code: String,
    state: State,
}

impl ContractInteract {
    async fn new() -> Self {
        let mut interactor = Interactor::new(GATEWAY).await;

        interactor.set_current_dir_from_workspace("esdt-safe/interactor");

        let wallet_address = interactor.register_wallet(test_wallets::bob()).await;
        let frank_address = interactor.register_wallet(test_wallets::frank()).await;
        let alice_address = interactor.register_wallet(test_wallets::alice()).await;
        let mike_address = interactor.register_wallet(test_wallets::mike()).await;
        let judy_address = interactor.register_wallet(test_wallets::judy()).await;

        ContractInteract {
            interactor,
            wallet_address,
            frank_address,
            alice_address,
            mike_address,
            judy_address,
            esdt_safe_code: ESDT_SAFE_CODE_PATH.to_string(),
            header_verifier_code: HEADER_VERIFIER_CODE_PATH.to_string(),
            fee_market_code: FEE_MARKET_CODE_PATH.to_string(),
            testing_sc_code: TESTING_SC_CODE_PATH.to_string(),
            state: State::load_state(),
        }
    }

    async fn deploy(&mut self, is_sov_chain: bool) {
        let code_path = MxscPath::new(self.esdt_safe_code.as_ref());

        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(110_000_000u64)
            .typed(EsdtSafeProxy)
            .init(is_sov_chain)
            .code(code_path)
            .returns(ReturnsNewAddress)
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

        let fee_market_code_path = MxscPath::new(&self.fee_market_code);
        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(100_000_000u64)
            .typed(FeeMarketProxy)
            .init(self.state.current_address(), Option::Some(fee))
            .code(fee_market_code_path)
            .returns(ReturnsNewAddress)
            .run()
            .await;

        let new_address_bech32 = bech32::encode(&new_address);
        self.state
            .set_fee_market_address(Bech32Address::from_bech32_string(
                new_address_bech32.clone(),
            ));
        println!("new fee_market_address: {new_address_bech32}");
    }

    async fn deploy_header_verifier_contract(&mut self) {
        let header_verifier_code_path = MxscPath::new(&self.header_verifier_code);

        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(100_000_000u64)
            .typed(HeaderverifierProxy)
            .init(MultiValueEncoded::new())
            .code(header_verifier_code_path)
            .returns(ReturnsNewAddress)
            .run()
            .await;

        let new_address_bech32 = bech32::encode(&new_address);
        self.state
            .set_header_verifier_address(Bech32Address::from_bech32_string(
                new_address_bech32.clone(),
            ));

        println!("new header_verifier_address: {new_address_bech32}");
    }

    async fn deploy_testing_contract(&mut self) {
        let testing_sc_code_path = MxscPath::new(&self.testing_sc_code);

        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(100_000_000u64)
            .typed(TestingScProxy)
            .init()
            .code(testing_sc_code_path)
            .returns(ReturnsNewAddress)
            .run()
            .await;

        let new_address_bech32 = bech32::encode(&new_address);
        self.state
            .set_testing_sc_address(Bech32Address::from_bech32_string(
                new_address_bech32.clone(),
            ));

        println!("new testing_sc_address: {new_address_bech32}");
    }

    async fn call_hello_endpoint(&mut self, value: u64) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(&self.state.get_testing_sc_address())
            .gas(50_000_000u64)
            .typed(TestingScProxy)
            .hello(value)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn upgrade(&mut self) {
        let code_path = MxscPath::new(&self.esdt_safe_code);

        let response = self
            .interactor
            .tx()
            .to(self.state.current_address())
            .from(&self.wallet_address)
            .gas(30_000_000u64)
            .typed(EsdtSafeProxy)
            .upgrade()
            .code(code_path)
            .code_metadata(CodeMetadata::UPGRADEABLE)
            .returns(ReturnsNewAddress)
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
            .typed(EsdtSafeProxy)
            .set_fee_market_address(fee_market_address)
            .returns(ReturnsResultUnmanaged)
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
            .typed(EsdtSafeProxy)
            .set_header_verifier_address(header_verifier_address)
            .returns(ReturnsResultUnmanaged)
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

        let to = &self.frank_address;
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
                    .typed(EsdtSafeProxy)
                    .deposit(to, transfer_data)
                    .payment(payments)
                    .returns(error)
                    .run()
                    .await;
            }
            None => {
                self.interactor
                    .tx()
                    .from(&self.wallet_address)
                    .to(self.state.current_address())
                    .gas(90_000_000u64)
                    .typed(EsdtSafeProxy)
                    .deposit(to, transfer_data)
                    .payment(payments)
                    .returns(ReturnsResultUnmanaged)
                    .run()
                    .await;
            }
        }
    }

    async fn register_token(&mut self) {
        let egld_amount = BigUint::<StaticApi>::from(50_000_000_000_000_000u64);

        let sov_token_id = TokenIdentifier::from_esdt_bytes(b"x-SOV-101252");
        let token_type = EsdtTokenType::Fungible;
        let token_display_name = ManagedBuffer::new_from_bytes(b"TESDT");
        let token_ticker = ManagedBuffer::new_from_bytes(b"TEST");
        let num_decimals = 18u32;

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(90_000_000u64)
            .typed(EsdtSafeProxy)
            .register_token(
                sov_token_id,
                token_type,
                token_display_name,
                token_ticker,
                num_decimals,
            )
            .egld(egld_amount)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn execute_operations(
        &mut self,
        operation: &Operation<StaticApi>,
        expect_error: Option<TxResponseStatus>,
    ) {
        let hash_of_hashes = sha256(&self.get_operation_hash(operation));

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(70_000_000u64)
            .typed(EsdtSafeProxy)
            .execute_operations(&hash_of_hashes, operation)
            .returns(ReturnsHandledOrError::new().returns(ReturnsResultUnmanaged))
            .run()
            .await;

        if let Err(err) = response {
            assert!(err == expect_error.unwrap());
        }
    }

    async fn execute_operations_with_error(&mut self, error_msg: ExpectError<'_>) {
        let tokens = self.setup_payments().await;
        let operation_data = self.setup_operation_data(false).await;
        let to = managed_address!(&self.frank_address);
        let operation = Operation::new(to, tokens, operation_data);
        let operation_hash = self.get_operation_hash(&operation);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(30_000_000u64)
            .typed(EsdtSafeProxy)
            .execute_operations(&operation_hash, operation)
            .returns(error_msg)
            .run()
            .await;

        println!("Result: {response:?}");
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
            .typed(EsdtSafeProxy)
            .set_max_bridged_amount(token_id, max_amount)
            .returns(ReturnsResultUnmanaged)
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
            .typed(EsdtSafeProxy)
            .max_bridged_amount(token_id)
            .returns(ReturnsResultUnmanaged)
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
            .typed(EsdtSafeProxy)
            .end_setup_phase()
            .returns(ReturnsResultUnmanaged)
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
            .typed(EsdtSafeProxy)
            .add_tokens_to_whitelist(tokens)
            .returns(ReturnsResultUnmanaged)
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
            .typed(EsdtSafeProxy)
            .remove_tokens_from_whitelist(tokens)
            .returns(ReturnsResultUnmanaged)
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
            .typed(EsdtSafeProxy)
            .add_tokens_to_blacklist(tokens)
            .returns(ReturnsResultUnmanaged)
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
            .typed(EsdtSafeProxy)
            .remove_tokens_from_blacklist(tokens)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn token_whitelist(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(EsdtSafeProxy)
            .token_whitelist()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn token_blacklist(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(EsdtSafeProxy)
            .token_blacklist()
            .returns(ReturnsResultUnmanaged)
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
            .typed(EsdtSafeProxy)
            .pause_endpoint()
            .returns(ReturnsResultUnmanaged)
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
            .typed(EsdtSafeProxy)
            .unpause_endpoint()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn paused_status(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(EsdtSafeProxy)
            .paused_status()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    async fn remove_fee(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.get_fee_market_address())
            .gas(30_000_000u64)
            .typed(FeeMarketProxy)
            .remove_fee(TOKEN_ID)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn header_verifier_set_esdt_address(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.get_header_verifier_address())
            .gas(30_000_000u64)
            .typed(HeaderverifierProxy)
            .set_esdt_safe_address(self.state.current_address())
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn setup_operation(&mut self, has_transfer_data: bool) -> Operation<StaticApi> {
        let to = managed_address!(&self.state.get_testing_sc_address());
        let payments = self.setup_payments().await;

        let operation_data = self.setup_operation_data(has_transfer_data).await;

        Operation::new(to, payments, operation_data)
    }

    async fn setup_operation_data(&mut self, has_transfer_data: bool) -> OperationData<StaticApi> {
        let op_sender = managed_address!(&self.wallet_address);

        let transfer_data = if has_transfer_data {
            let mut args = ManagedVec::new();
            let value = BigUint::<StaticApi>::from(0u64);
            args.push(ManagedBuffer::from(value.to_bytes_be()));

            Some(TransferData::new(
                30_000_000u64,
                ManagedBuffer::from("hello"),
                args,
            ))
        } else {
            None
        };

        let operation_data: OperationData<StaticApi> = OperationData {
            op_nonce: 1,
            op_sender,
            opt_transfer_data: transfer_data,
        };

        operation_data
    }

    async fn register_operations(&mut self, operation: &Operation<StaticApi>) {
        let bls_signature = ManagedBuffer::new();
        let operation_hash = self.get_operation_hash(operation);
        let hash_of_hashes = sha256(&operation_hash);

        let mut managed_operation_hashes =
            MultiValueEncoded::<StaticApi, ManagedBuffer<StaticApi>>::new();

        let managed_operation_hash = ManagedBuffer::<StaticApi>::from(&operation_hash);
        let managed_hash_of_hashes = ManagedBuffer::<StaticApi>::from(&hash_of_hashes);

        managed_operation_hashes.push(managed_operation_hash);

        let header_verifier_address = self.state.get_header_verifier_address();

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(header_verifier_address)
            .typed(HeaderverifierProxy)
            .register_bridge_operations(
                bls_signature,
                managed_hash_of_hashes,
                managed_operation_hashes,
            )
            .returns(ReturnsResult)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn setup_payments(&mut self) -> ManagedVec<StaticApi, OperationEsdtPayment<StaticApi>> {
        let mut tokens: ManagedVec<StaticApi, OperationEsdtPayment<StaticApi>> = ManagedVec::new();
        let token_ids = vec![TOKEN_ID_FOR_EXECUTE];

        for token_id in token_ids {
            let payment: OperationEsdtPayment<StaticApi> = OperationEsdtPayment {
                token_identifier: token_id.into(),
                token_nonce: 0,
                token_data: EsdtTokenData {
                    token_type: EsdtTokenType::Fungible,
                    amount: BigUint::from(10_000u64),
                    frozen: false,
                    hash: ManagedBuffer::new(),
                    name: ManagedBuffer::from("SovToken"),
                    attributes: ManagedBuffer::new(),
                    creator: managed_address!(&self.frank_address),
                    royalties: BigUint::zero(),
                    uris: ManagedVec::new(),
                },
            };

            tokens.push(payment);
        }

        tokens
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
#[ignore]
async fn test_deploy_sov() {
    let mut interact = ContractInteract::new().await;
    interact.deploy(false).await;
    interact.deploy_fee_market().await;
    interact.set_fee_market_address().await;
    interact.remove_fee().await;
    interact.deploy_header_verifier_contract().await;
    interact.set_header_verifier_address().await;
    interact.unpause_endpoint().await;
    interact.header_verifier_set_esdt_address().await;
    interact.deploy_testing_contract().await;
    interact.register_token().await;

    let operation = interact.setup_operation(true).await;
    interact.register_operations(&operation).await;
    interact
        .execute_operations(
            &operation,
            Some(TxResponseStatus::new(
                ReturnCode::UserError,
                "Value should be greater than 0",
            )),
        )
        .await;
}
