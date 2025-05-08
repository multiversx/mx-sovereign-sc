#![allow(non_snake_case)]
#![allow(unused)]

use common_interactor::common_interactor_sovereign::CommonInteractorTrait;
use common_interactor::constants::TOKEN_ID;
use common_interactor::interactor_config::Config;
use common_interactor::interactor_state::State;
use common_test_setup::constants::ENSHRINE_ESDT_SAFE_CODE_PATH;
use fee_market_proxy::*;
use multiversx_sc_snippets::imports::*;
use proxies::*;
use structs::aliases::{OptionalTransferData, PaymentsVec};
use structs::configs::EsdtSafeConfig;
use structs::operation::{Operation, OperationData};

pub struct EnshrineEsdtSafeInteract {
    pub interactor: Interactor,
    pub wallet_address: Address,
    pub bob_address: Address,
    pub alice_address: Address,
    pub mike_address: Address,
    pub judy_address: Address,
    pub state: State,
}

impl CommonInteractorTrait for EnshrineEsdtSafeInteract {
    fn interactor(&mut self) -> &mut Interactor {
        &mut self.interactor
    }

    fn state(&mut self) -> &mut State {
        &mut self.state
    }

    fn wallet_address(&mut self) -> &Address {
        &self.wallet_address
    }
}

impl EnshrineEsdtSafeInteract {
    pub async fn new(config: Config) -> Self {
        let mut interactor = Interactor::new(config.gateway_uri())
            .await
            .use_chain_simulator(config.use_chain_simulator());
        interactor.set_current_dir_from_workspace("interactor");

        let wallet_address = interactor.register_wallet(test_wallets::frank()).await;
        let bob_address = interactor.register_wallet(test_wallets::bob()).await;
        let alice_address = interactor.register_wallet(test_wallets::alice()).await;
        let mike_address = interactor.register_wallet(test_wallets::mike()).await;
        let judy_address = interactor.register_wallet(test_wallets::judy()).await;

        EnshrineEsdtSafeInteract {
            interactor,
            wallet_address,
            bob_address,
            alice_address,
            mike_address,
            judy_address,
            state: State::load_state(),
        }
    }

    pub async fn deploy_all(
        &mut self,
        is_sov_chain: bool,
        opt_config: Option<EsdtSafeConfig<StaticApi>>,
    ) {
        self.deploy_chain_factory().await;
        self.deploy_token_handler().await;
        self.deploy_enshrine_esdt(is_sov_chain, opt_config).await;
        self.deploy_header_verifier().await;
        self.deploy_fee_market(None).await;
        self.unpause_endpoint().await;
    }

    pub async fn deploy_setup(&mut self, opt_config: Option<EsdtSafeConfig<StaticApi>>) {
        self.deploy_token_handler().await;
        self.deploy_enshrine_esdt(false, opt_config).await;
        self.unpause_endpoint().await;
    }

    pub async fn upgrade(&mut self) {
        let response = self
            .interactor
            .tx()
            .to(self.state.current_enshrine_esdt_safe_address())
            .from(&self.wallet_address)
            .gas(30_000_000u64)
            .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
            .upgrade()
            .code(ENSHRINE_ESDT_SAFE_CODE_PATH)
            .code_metadata(CodeMetadata::all())
            .returns(ReturnsNewAddress)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn set_fee_market_address_in_enshrine_esdt_safe(&mut self) {
        let fee_market_address = self.state.current_fee_market_address();

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_enshrine_esdt_safe_address())
            .gas(30_000_000u64)
            .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
            .set_fee_market_address(fee_market_address)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn set_header_verifier_address_in_enshrine_esdt_safe(&mut self) {
        let header_verifier_address = self.state.current_header_verifier_address();

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_enshrine_esdt_safe_address())
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
        error_wanted: Option<&str>,
    ) {
        let token_id = TOKEN_ID;
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(20u64);
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

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_enshrine_esdt_safe_address())
            .gas(30_000_000u64)
            .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
            .deposit(to, transfer_data)
            .payment(payments)
            .returns(ReturnsHandledOrError::new())
            .run()
            .await;

        self.assert_expected_error_message(response, error_wanted);
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
            .to(self.state.current_enshrine_esdt_safe_address())
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
            .to(self.state.current_enshrine_esdt_safe_address())
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
            .to(self.state.current_enshrine_esdt_safe_address())
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
            .to(self.state.current_enshrine_esdt_safe_address())
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
            .to(self.state.current_enshrine_esdt_safe_address())
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
            .to(self.state.current_enshrine_esdt_safe_address())
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
            .to(self.state.current_enshrine_esdt_safe_address())
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
            .to(self.state.current_enshrine_esdt_safe_address())
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
            .to(self.state.current_enshrine_esdt_safe_address())
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
            .to(self.state.current_enshrine_esdt_safe_address())
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
            .to(self.state.current_enshrine_esdt_safe_address())
            .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
            .paused_status()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }
}
