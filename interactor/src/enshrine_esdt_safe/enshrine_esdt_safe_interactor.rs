#![allow(non_snake_case)]
#![allow(unused)]

use common_interactor::common_sovereign_interactor::CommonInteractorTrait;
use common_interactor::constants::TOKEN_ID;
use common_interactor::interactor_config::Config;
use common_interactor::interactor_state::State;
use common_test_setup::constants::ENSHRINE_ESDT_SAFE_CODE_PATH;
use common_test_setup::RegisterTokenArgs;
use fee_market_proxy::*;
use multiversx_sc_snippets::imports::*;
use proxies::enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy;
use proxies::*;
use structs::aliases::{OptionalTransferData, PaymentsVec};
use structs::configs::EsdtSafeConfig;
use structs::operation::{self, Operation, OperationData};

use crate::sovereign_forge;

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

    pub async fn upgrade(&mut self) {
        let response = self
            .interactor
            .tx()
            .to(self.state.current_enshrine_esdt_safe_address())
            .from(&self.wallet_address)
            .gas(30_000_000u64)
            .typed(EnshrineEsdtSafeProxy)
            .upgrade()
            .code(ENSHRINE_ESDT_SAFE_CODE_PATH)
            .code_metadata(CodeMetadata::all())
            .returns(ReturnsNewAddress)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn set_fee_market_address_in_enshrine_esdt_safe(
        &mut self,
        fee_market_address: Bech32Address,
    ) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_enshrine_esdt_safe_address())
            .gas(30_000_000u64)
            .typed(EnshrineEsdtSafeProxy)
            .set_fee_market_address(fee_market_address)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn set_header_verifier_address_in_enshrine_esdt_safe(
        &mut self,
        header_verifier_address: Bech32Address,
    ) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_enshrine_esdt_safe_address())
            .gas(30_000_000u64)
            .typed(EnshrineEsdtSafeProxy)
            .set_header_verifier_address(header_verifier_address)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn deposit(
        &mut self,
        payments: PaymentsVec<StaticApi>,
        to: Bech32Address,
        transfer_data: OptionalTransferData<StaticApi>,
        error_wanted: Option<&str>,
    ) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_enshrine_esdt_safe_address())
            .gas(30_000_000u64)
            .typed(EnshrineEsdtSafeProxy)
            .deposit(to, transfer_data)
            .payment(payments)
            .returns(ReturnsHandledOrError::new())
            .run()
            .await;

        self.assert_expected_error_message(response, error_wanted);
    }

    pub async fn execute_operations(
        &mut self,
        hash_of_hashes: ManagedBuffer<StaticApi>,
        operation: Operation<StaticApi>,
    ) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_enshrine_esdt_safe_address())
            .gas(30_000_000u64)
            .typed(EnshrineEsdtSafeProxy)
            .execute_operations(hash_of_hashes, operation)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn register_new_token_id(
        &mut self,
        payment_token_id: TokenIdentifier<StaticApi>,
        payment_token_nonce: u64,
        payment_amount: BigUint<StaticApi>,
        token_identifiers: MultiValueEncoded<StaticApi, TokenIdentifier<StaticApi>>,
    ) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_enshrine_esdt_safe_address())
            .gas(30_000_000u64)
            .typed(EnshrineEsdtSafeProxy)
            .register_new_token_id(token_identifiers)
            .payment((payment_token_id, payment_token_nonce, payment_amount))
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn add_tokens_to_whitelist(
        &mut self,
        tokens: MultiValueVec<TokenIdentifier<StaticApi>>,
    ) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_enshrine_esdt_safe_address())
            .gas(30_000_000u64)
            .typed(EnshrineEsdtSafeProxy)
            .add_tokens_to_whitelist(tokens)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn remove_tokens_from_whitelist(
        &mut self,
        tokens: MultiValueVec<TokenIdentifier<StaticApi>>,
    ) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_enshrine_esdt_safe_address())
            .gas(30_000_000u64)
            .typed(EnshrineEsdtSafeProxy)
            .remove_tokens_from_whitelist(tokens)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn add_tokens_to_blacklist(
        &mut self,
        tokens: MultiValueVec<TokenIdentifier<StaticApi>>,
    ) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_enshrine_esdt_safe_address())
            .gas(30_000_000u64)
            .typed(EnshrineEsdtSafeProxy)
            .add_tokens_to_blacklist(tokens)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn remove_tokens_from_blacklist(
        &mut self,
        tokens: MultiValueVec<TokenIdentifier<StaticApi>>,
    ) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_enshrine_esdt_safe_address())
            .gas(30_000_000u64)
            .typed(EnshrineEsdtSafeProxy)
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
            .typed(EnshrineEsdtSafeProxy)
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
            .typed(EnshrineEsdtSafeProxy)
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
            .typed(EnshrineEsdtSafeProxy)
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
            .typed(EnshrineEsdtSafeProxy)
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
            .typed(EnshrineEsdtSafeProxy)
            .paused_status()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }
}
