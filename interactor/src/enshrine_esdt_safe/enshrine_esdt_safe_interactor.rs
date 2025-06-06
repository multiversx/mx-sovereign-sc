#![allow(non_snake_case)]
#![allow(unused)]

use common_interactor::common_sovereign_interactor::{
    CommonInteractorTrait, IssueTokenStruct, MintTokenStruct,
};
use common_interactor::interactor_config::Config;
use common_interactor::interactor_state::State;
use common_test_setup::constants::{
    DEPLOY_COST, ENSHRINE_ESDT_SAFE_CODE_PATH, INTERACTOR_WORKING_DIR, ONE_THOUSAND_TOKENS,
    SOVEREIGN_TOKEN_PREFIX,
};
use common_test_setup::RegisterTokenArgs;
use fee_market_proxy::*;
use multiversx_sc_snippets::imports::*;
use proxies::enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy;
use proxies::*;
use structs::aliases::{OptionalTransferData, PaymentsVec};
use structs::configs::{EsdtSafeConfig, SovereignConfig};
use structs::fee::FeeStruct;
use structs::forge::ScArray;
use structs::operation::{self, Operation, OperationData};

use crate::sovereign_forge;

pub struct EnshrineEsdtSafeInteract {
    pub interactor: Interactor,
    pub owner_address: Address,
    pub user_address: Address,
    pub state: State,
}

impl CommonInteractorTrait for EnshrineEsdtSafeInteract {
    fn interactor(&mut self) -> &mut Interactor {
        &mut self.interactor
    }

    fn state(&mut self) -> &mut State {
        &mut self.state
    }

    fn owner_address(&self) -> &Address {
        &self.owner_address
    }

    fn user_address(&self) -> &Address {
        &self.user_address
    }
}

impl EnshrineEsdtSafeInteract {
    pub async fn new(config: Config) -> Self {
        let mut interactor = Self::initialize_interactor(config).await;
        interactor.initialize_tokens_in_wallets().await;
        interactor
    }

    async fn initialize_interactor(config: Config) -> Self {
        let mut interactor = Interactor::new(config.gateway_uri())
            .await
            .use_chain_simulator(config.use_chain_simulator());

        let working_dir = INTERACTOR_WORKING_DIR;
        interactor.set_current_dir_from_workspace(working_dir);
        let owner_address = interactor.register_wallet(test_wallets::mike()).await;
        let user_address = interactor.register_wallet(test_wallets::bob()).await;

        interactor.generate_blocks_until_epoch(1u64).await.unwrap();

        EnshrineEsdtSafeInteract {
            interactor,
            owner_address,
            user_address,
            state: State::load_state(),
        }
    }

    async fn initialize_tokens_in_wallets(&mut self) {
        let first_token_struct = IssueTokenStruct {
            token_display_name: "ESH".to_string(),
            token_ticker: "ESH".to_string(),
            token_type: EsdtTokenType::Fungible,
            num_decimals: 18,
        };
        let first_token_mint = MintTokenStruct {
            name: None,
            amount: BigUint::from(ONE_THOUSAND_TOKENS),
            attributes: None,
        };
        let first_token = self
            .issue_and_mint_token(first_token_struct, first_token_mint)
            .await;
        self.state.set_first_token(first_token);

        let second_token_struct = IssueTokenStruct {
            token_display_name: "ESH2".to_string(),
            token_ticker: "ESH2".to_string(),
            token_type: EsdtTokenType::Fungible,
            num_decimals: 18,
        };
        let second_token_mint = MintTokenStruct {
            name: None,
            amount: BigUint::from(ONE_THOUSAND_TOKENS),
            attributes: None,
        };
        let second_token = self
            .issue_and_mint_token(second_token_struct, second_token_mint)
            .await;
        self.state.set_second_token(second_token);

        let fee_token_mint = MintTokenStruct {
            name: None,
            amount: BigUint::from(ONE_THOUSAND_TOKENS),
            attributes: None,
        };
        let fee_token_struct = IssueTokenStruct {
            token_display_name: "FEE".to_string(),
            token_ticker: "FEE".to_string(),
            token_type: EsdtTokenType::Fungible,
            num_decimals: 18,
        };
        let fee_token = self
            .issue_and_mint_token(fee_token_struct, fee_token_mint)
            .await;
        self.state.set_fee_token(fee_token);
    }

    //TODO: The unpause should be done via the chain factory, will refactor in the future
    pub async fn deploy_contracts(
        &mut self,
        is_sovereign_chain: bool,
        fee_struct: Option<FeeStruct<StaticApi>>,
        opt_config: Option<EsdtSafeConfig<StaticApi>>,
        sc_array: Vec<ScArray>,
    ) {
        let owner = self.owner_address().clone();
        self.deploy_chain_config(OptionalValue::None).await;
        self.deploy_token_handler(owner).await;
        self.deploy_enshrine_esdt(
            is_sovereign_chain,
            Some(self.state.get_first_token_id()),
            Some(SOVEREIGN_TOKEN_PREFIX.into()),
            self.state.current_token_handler_address().clone(),
            opt_config,
        )
        .await;
        self.whitelist_enshrine_esdt(self.state.current_enshrine_esdt_safe_address().clone())
            .await;
        self.deploy_fee_market(
            self.state.current_enshrine_esdt_safe_address().clone(),
            fee_struct,
        )
        .await;
        self.set_fee_market_address_in_enshrine_esdt_safe(
            self.state.current_fee_market_address().clone(),
        )
        .await;
        let contracts_array = self.get_contract_info_struct_for_sc_type(sc_array);

        self.deploy_header_verifier(contracts_array).await;
        self.complete_header_verifier_setup_phase().await;
        self.unpause_endpoint().await;
    }

    pub async fn upgrade(&mut self) {
        let response = self
            .interactor
            .tx()
            .to(self.state.current_enshrine_esdt_safe_address())
            .from(&self.owner_address)
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
            .from(&self.owner_address)
            .to(self.state.current_enshrine_esdt_safe_address())
            .gas(30_000_000u64)
            .typed(EnshrineEsdtSafeProxy)
            .set_fee_market_address(fee_market_address)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn deposit(
        &mut self,
        payments: PaymentsVec<StaticApi>,
        to: Address,
        transfer_data: OptionalTransferData<StaticApi>,
        error_wanted: Option<&str>,
        expected_log: Option<&str>,
    ) {
        let (response, logs) = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .to(self.state.current_enshrine_esdt_safe_address())
            .gas(30_000_000u64)
            .typed(EnshrineEsdtSafeProxy)
            .deposit(to, transfer_data)
            .payment(payments)
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run()
            .await;

        self.assert_expected_error_message(response, error_wanted);

        self.assert_expected_log(logs, expected_log);
    }

    pub async fn execute_operation(
        &mut self,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        operation: Operation<StaticApi>,
        expected_error_message: Option<&str>,
        expected_log: Option<&str>,
    ) {
        let (response, logs) = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .to(self.state.current_enshrine_esdt_safe_address())
            .gas(30_000_000u64)
            .typed(EnshrineEsdtSafeProxy)
            .execute_operations(hash_of_hashes, operation)
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run()
            .await;

        self.assert_expected_error_message(response, expected_error_message);

        self.assert_expected_log(logs, expected_log);
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
            .from(&self.owner_address)
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

    pub async fn register_tokens(
        &mut self,
        fee_payment: EsdtTokenPayment<StaticApi>,
        tokens_to_register: Vec<TokenIdentifier<StaticApi>>,
        error_message: Option<&str>,
    ) {
        let mut managed_token_ids: MultiValueEncoded<StaticApi, TokenIdentifier<StaticApi>> =
            MultiValueEncoded::from_iter(tokens_to_register);

        let response = self
            .interactor
            .tx()
            .from(self.owner_address.clone())
            .to(self.state.current_enshrine_esdt_safe_address())
            .gas(30_000_000u64)
            .typed(EnshrineEsdtSafeProxy)
            .register_new_token_id(managed_token_ids)
            .esdt(fee_payment)
            .returns(ReturnsHandledOrError::new())
            .run()
            .await;

        self.assert_expected_error_message(response, error_message);
    }

    pub async fn add_tokens_to_whitelist(
        &mut self,
        tokens: MultiValueVec<TokenIdentifier<StaticApi>>,
    ) {
        let response = self
            .interactor
            .tx()
            .from(&self.owner_address)
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
            .from(&self.owner_address)
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
            .from(&self.owner_address)
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
            .from(&self.owner_address)
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
            .from(&self.owner_address)
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
            .from(&self.owner_address)
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
