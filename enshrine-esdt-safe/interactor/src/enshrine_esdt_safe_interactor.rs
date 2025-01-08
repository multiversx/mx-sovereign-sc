#![allow(non_snake_case)]
#![allow(unused)]

use aliases::{OptionalTransferData, PaymentsVec};
use fee_market_proxy::*;
use interactor::constants::{TOKEN_ID, WHITELIST_TOKEN_ID};
use interactor::interactor_config::Config;
use interactor::interactor_state::State;
use multiversx_sc_snippets::imports::*;
use operation::*;
use proxies::*;

const FEE_MARKET_CODE_PATH: &str = "../fee-market/output/fee-market.mxsc.json";
const HEADER_VERIFIER_CODE_PATH: &str = "../header-verifier/output/header-verifier.mxsc.json";
const ENSHRINE_ESDT_SAFE_CODE_PATH: &str = "output/enshrine-esdt-safe.mxsc.json";
const TOKEN_HANDLER_CODE_PATH: &str = "../token-handler/output/token-handler.mxsc.json";

pub async fn enshrine_esdt_safe_cli() {
    env_logger::init();

    let mut args = std::env::args();
    let _ = args.next();
    let cmd = args.next().expect("at least one argument required");

    let config = Config::load_config();
    let mut interact = ContractInteract::new(config).await;
    match cmd.as_str() {
        "deploy" => interact.deploy(false, None).await,
        "upgrade" => interact.upgrade().await,
        "setFeeMarketAddress" => interact.set_fee_market_address().await,
        "setHeaderVerifierAddress" => interact.set_header_verifier_address().await,
        "deposit" => interact.deposit(None.into(), Option::None).await,
        "executeBridgeOps" => interact.execute_operations().await,
        "registerNewTokenID" => interact.register_new_token_id().await,
        "setMaxBridgedAmount" => interact.set_max_bridged_amount().await,
        "getMaxBridgedAmount" => interact.max_bridged_amount().await,
        "addTokensToWhitelist" => interact.add_tokens_to_whitelist(TOKEN_ID).await,
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

pub struct ContractInteract {
    pub interactor: Interactor,
    pub wallet_address: Address,
    pub bob_address: Address,
    pub alice_address: Address,
    pub mike_address: Address,
    pub judy_address: Address,
    enshrine_esdt_safe_code: String,
    token_handler_code: String,
    fee_market_code: String,
    header_verifier_code: String,
    pub state: State,
}

impl ContractInteract {
    pub async fn new(config: Config) -> Self {
        let mut interactor = Interactor::new(config.gateway_uri())
            .await
            .use_chain_simulator(config.use_chain_simulator());
        interactor.set_current_dir_from_workspace("enshrine-esdt-safe");

        let wallet_address = interactor.register_wallet(test_wallets::frank()).await;
        let bob_address = interactor.register_wallet(test_wallets::bob()).await;
        let alice_address = interactor.register_wallet(test_wallets::alice()).await;
        let mike_address = interactor.register_wallet(test_wallets::mike()).await;
        let judy_address = interactor.register_wallet(test_wallets::judy()).await;

        ContractInteract {
            interactor,
            wallet_address,
            bob_address,
            alice_address,
            mike_address,
            judy_address,
            enshrine_esdt_safe_code: ENSHRINE_ESDT_SAFE_CODE_PATH.to_string(),
            token_handler_code: TOKEN_HANDLER_CODE_PATH.to_string(),
            fee_market_code: FEE_MARKET_CODE_PATH.to_string(),
            header_verifier_code: HEADER_VERIFIER_CODE_PATH.to_string(),
            state: State::load_state(),
        }
    }

    pub async fn deploy(
        &mut self,
        is_sovereign_chain: bool,
        opt_config: Option<BridgeConfig<StaticApi>>,
    ) {
        let opt_wegld_identifier =
            Option::Some(TokenIdentifier::from_esdt_bytes(WHITELIST_TOKEN_ID));
        let opt_sov_token_prefix = Option::Some(ManagedBuffer::new_from_bytes(&b"sov"[..]));
        let token_handler_address =
            managed_address!(self.state.get_token_handler_address().as_address());

        let code_path = MxscPath::new(self.enshrine_esdt_safe_code.as_ref());
        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(100_000_000u64)
            .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
            .init(
                is_sovereign_chain,
                token_handler_address,
                opt_wegld_identifier,
                opt_sov_token_prefix,
                opt_config,
            )
            .code(code_path)
            .returns(ReturnsNewAddress)
            .run()
            .await;
        let new_address_bech32 = bech32::encode(&new_address);
        self.state
            .set_esdt_safe_address(Bech32Address::from_bech32_string(
                new_address_bech32.clone(),
            ));

        println!("new address: {new_address_bech32}");
    }

    pub async fn deploy_header_verifier(&mut self) {
        let bls_pub_key: ManagedBuffer<StaticApi> = ManagedBuffer::new();
        let mut bls_pub_keys = MultiValueEncoded::new();
        bls_pub_keys.push(bls_pub_key);
        let header_verifier_code_path = MxscPath::new(&self.header_verifier_code);

        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(100_000_000u64)
            .typed(header_verifier_proxy::HeaderverifierProxy)
            .init(bls_pub_keys)
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

    pub async fn deploy_fee_market(&mut self) {
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
            .typed(fee_market_proxy::FeeMarketProxy)
            .init(self.state.esdt_safe_address(), Option::Some(fee))
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

    pub async fn deploy_token_handler(&mut self) {
        let token_handler_code_path = MxscPath::new(&self.token_handler_code);
        let chain_factory_address = Bech32Address::from_bech32_string("chain_factory".to_string());

        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(100_000_000u64)
            .typed(token_handler_proxy::TokenHandlerProxy)
            .init(chain_factory_address)
            .code(token_handler_code_path)
            .returns(ReturnsNewAddress)
            .run()
            .await;
        let new_address_bech32 = bech32::encode(&new_address);
        self.state
            .set_token_handler_address(Bech32Address::from_bech32_string(
                new_address_bech32.clone(),
            ));
        println!("new token_handler_address: {new_address_bech32}");
    }

    pub async fn deploy_all(
        &mut self,
        is_sov_chain: bool,
        opt_config: Option<BridgeConfig<StaticApi>>,
    ) {
        self.deploy_token_handler().await;
        self.deploy(is_sov_chain, opt_config).await;
        self.deploy_header_verifier().await;
        self.deploy_fee_market().await;
        self.unpause_endpoint().await;
    }

    pub async fn deploy_setup(&mut self, opt_config: Option<BridgeConfig<StaticApi>>) {
        self.deploy_token_handler().await;
        self.deploy(false, opt_config).await;
        self.unpause_endpoint().await;
    }

    pub async fn upgrade(&mut self) {
        let code_path = MxscPath::new(&self.enshrine_esdt_safe_code);
        let response = self
            .interactor
            .tx()
            .to(self.state.esdt_safe_address())
            .from(&self.wallet_address)
            .gas(30_000_000u64)
            .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
            .upgrade()
            .code(code_path)
            .code_metadata(CodeMetadata::UPGRADEABLE)
            .returns(ReturnsNewAddress)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn set_fee_market_address(&mut self) {
        let fee_market_address = self.state.get_fee_market_address();

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.esdt_safe_address())
            .gas(30_000_000u64)
            .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
            .set_fee_market_address(fee_market_address)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn set_header_verifier_address(&mut self) {
        let header_verifier_address = self.state.get_header_verifier_address();

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.esdt_safe_address())
            .gas(30_000_000u64)
            .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
            .set_header_verifier_address(header_verifier_address)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn deposit(
        &mut self,
        transfer_data: OptionalTransferData<StaticApi>,
        error_wanted: Option<ExpectError<'_>>,
    ) {
        let token_id = TOKEN_ID;
        let token_nonce = 0u64;
        let token_amount = BigUint::from(20u64);
        let to = &self.bob_address;
        let mut payments = PaymentsVec::new();
        payments.push(EsdtTokenPayment::new(
            TokenIdentifier::from(token_id),
            token_nonce,
            token_amount.clone(),
        ));
        payments.push(EsdtTokenPayment::new(
            TokenIdentifier::from(token_id),
            token_nonce,
            BigUint::from(30u64),
        ));

        match error_wanted {
            Some(error) => {
                self.interactor
                    .tx()
                    .from(&self.wallet_address)
                    .to(self.state.esdt_safe_address())
                    .gas(30_000_000u64)
                    .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
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
                    .to(self.state.esdt_safe_address())
                    .gas(30_000_000u64)
                    .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
                    .deposit(to, transfer_data)
                    .payment(payments)
                    .returns(ReturnsResultUnmanaged)
                    .run()
                    .await;
            }
        }
    }
    pub async fn execute_operations(&mut self) {
        let hash_of_hashes = ManagedBuffer::new_from_bytes(&b""[..]);
        let operation = Operation::new(
            ManagedAddress::zero(),
            ManagedVec::new(),
            OperationData::new(0, ManagedAddress::zero(), Option::None),
        );

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.esdt_safe_address())
            .gas(30_000_000u64)
            .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
            .execute_operations(hash_of_hashes, operation)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn register_new_token_id(&mut self) {
        let token_id = String::new();
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(0u128);

        let tokens = MultiValueVec::from(vec![TokenIdentifier::from_esdt_bytes(&b""[..])]);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.esdt_safe_address())
            .gas(30_000_000u64)
            .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
            .register_new_token_id(tokens)
            .payment((
                TokenIdentifier::from(token_id.as_str()),
                token_nonce,
                token_amount,
            ))
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn set_max_bridged_amount(&mut self) {
        let token_id = TokenIdentifier::from_esdt_bytes(&b""[..]);
        let max_amount = BigUint::<StaticApi>::from(0u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.esdt_safe_address())
            .gas(30_000_000u64)
            .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
            .set_max_bridged_amount(token_id, max_amount)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn max_bridged_amount(&mut self) {
        let token_id = TokenIdentifier::from_esdt_bytes(&b""[..]);

        let result_value = self
            .interactor
            .query()
            .to(self.state.esdt_safe_address())
            .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
            .max_bridged_amount(token_id)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    pub async fn add_tokens_to_whitelist(&mut self, token_id: &[u8]) {
        let tokens;

        match token_id {
            WHITELIST_TOKEN_ID => {
                tokens =
                    MultiValueVec::from(vec![TokenIdentifier::from_esdt_bytes(WHITELIST_TOKEN_ID)]);
            }
            TOKEN_ID => {
                tokens = MultiValueVec::from(vec![TokenIdentifier::from_esdt_bytes(TOKEN_ID)]);
            }
            _ => {
                tokens = MultiValueVec::from(vec![TokenIdentifier::from_esdt_bytes(&b""[..])]);
                println!("Token not in whitelist");
            }
        }

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.esdt_safe_address())
            .gas(30_000_000u64)
            .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
            .add_tokens_to_whitelist(tokens)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn remove_tokens_from_whitelist(&mut self) {
        let tokens = MultiValueVec::from(vec![TokenIdentifier::from_esdt_bytes(&b""[..])]);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.esdt_safe_address())
            .gas(30_000_000u64)
            .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
            .remove_tokens_from_whitelist(tokens)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn add_tokens_to_blacklist(&mut self) {
        let tokens = MultiValueVec::from(vec![TokenIdentifier::from_esdt_bytes(&b""[..])]);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.esdt_safe_address())
            .gas(30_000_000u64)
            .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
            .add_tokens_to_blacklist(tokens)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn remove_tokens_from_blacklist(&mut self) {
        let tokens = MultiValueVec::from(vec![TokenIdentifier::from_esdt_bytes(&b""[..])]);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.esdt_safe_address())
            .gas(30_000_000u64)
            .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
            .remove_tokens_from_blacklist(tokens)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn token_whitelist(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.esdt_safe_address())
            .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
            .token_whitelist()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    pub async fn token_blacklist(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.esdt_safe_address())
            .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
            .token_blacklist()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    pub async fn pause_endpoint(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.esdt_safe_address())
            .gas(30_000_000u64)
            .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
            .pause_endpoint()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn unpause_endpoint(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.esdt_safe_address())
            .gas(30_000_000u64)
            .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
            .unpause_endpoint()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn paused_status(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.esdt_safe_address())
            .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
            .paused_status()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }
}
