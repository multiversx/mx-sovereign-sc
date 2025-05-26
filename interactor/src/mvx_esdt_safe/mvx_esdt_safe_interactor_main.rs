use common_interactor::common_sovereign_interactor::{
    CommonInteractorTrait, IssueTokenStruct, MintTokenStruct,
};
use common_interactor::constants::ONE_THOUSAND_TOKENS;
use multiversx_sc_snippets::imports::*;
use proxies::header_verifier_proxy::HeaderverifierProxy;
use proxies::mvx_esdt_safe_proxy::MvxEsdtSafeProxy;
use structs::aliases::{OptionalValueTransferDataTuple, PaymentsVec};

use structs::configs::{EsdtSafeConfig, SovereignConfig};
use structs::fee::FeeStruct;
use structs::operation::Operation;

use common_interactor::interactor_config::Config;
use common_interactor::interactor_state::State;

use common_test_setup::constants::MVX_ESDT_SAFE_CODE_PATH;
use common_test_setup::RegisterTokenArgs;

pub struct MvxEsdtSafeInteract {
    pub interactor: Interactor,
    pub owner_address: Address,
    pub user_address: Address,
    pub state: State,
}
impl CommonInteractorTrait for MvxEsdtSafeInteract {
    fn interactor(&mut self) -> &mut Interactor {
        &mut self.interactor
    }

    fn state(&mut self) -> &mut State {
        &mut self.state
    }

    fn wallet_address(&mut self) -> &Address {
        &self.owner_address
    }
}

impl MvxEsdtSafeInteract {
    pub async fn new(config: Config) -> Self {
        let mut interactor = Self::initialize_interactor(config).await;
        interactor.initialize_tokens_in_wallets().await;
        interactor
    }

    async fn initialize_interactor(config: Config) -> Self {
        let mut interactor = Interactor::new(config.gateway_uri())
            .await
            .use_chain_simulator(config.use_chain_simulator());

        let working_dir = "interactor";
        interactor.set_current_dir_from_workspace(working_dir);
        let owner_address = interactor.register_wallet(test_wallets::mike()).await;
        let user_address = interactor.register_wallet(test_wallets::bob()).await;

        interactor.generate_blocks_until_epoch(1u64).await.unwrap();

        MvxEsdtSafeInteract {
            interactor,
            owner_address,
            user_address,
            state: State::load_state(),
        }
    }

    async fn initialize_tokens_in_wallets(&mut self) {
        let first_token_struct = IssueTokenStruct {
            token_display_name: "MVX".to_string(),
            token_ticker: "MVX".to_string(),
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

        let fee_token_struct = IssueTokenStruct {
            token_display_name: "FEE".to_string(),
            token_ticker: "FEE".to_string(),
            token_type: EsdtTokenType::Fungible,
            num_decimals: 0,
        };
        let fee_token_mint = MintTokenStruct {
            name: None,
            amount: BigUint::from(ONE_THOUSAND_TOKENS),
            attributes: None,
        };
        let fee_token = self
            .issue_and_mint_token(fee_token_struct, fee_token_mint)
            .await;
        self.state.set_fee_token(fee_token);

        let second_token_struct = IssueTokenStruct {
            token_display_name: "MVX2".to_string(),
            token_ticker: "MVX2".to_string(),
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
    }

    pub async fn issue_and_mint_all_types_of_tokens(&mut self) {
        let first_token_struct = IssueTokenStruct {
            token_display_name: "FUNG".to_string(),
            token_ticker: "FUNG".to_string(),
            token_type: EsdtTokenType::Fungible,
            num_decimals: 18,
        };

        let first_token_mint = MintTokenStruct {
            name: None,
            amount: BigUint::from(ONE_THOUSAND_TOKENS),
            attributes: None,
        };

        self.issue_and_mint_token(first_token_struct, first_token_mint)
            .await;

        let second_token_struct = IssueTokenStruct {
            token_display_name: "NFT".to_string(),
            token_ticker: "NFT".to_string(),
            token_type: EsdtTokenType::NonFungible,
            num_decimals: 0,
        };
        let second_token_mint = MintTokenStruct {
            name: Some("NFT".to_string()),
            amount: BigUint::from(1u64),
            attributes: None,
        };
        self.issue_and_mint_token(second_token_struct, second_token_mint)
            .await;

        let third_token_struct = IssueTokenStruct {
            token_display_name: "SFT".to_string(),
            token_ticker: "SFT".to_string(),
            token_type: EsdtTokenType::SemiFungible,
            num_decimals: 0,
        };
        let third_token_mint = MintTokenStruct {
            name: Some("SFT".to_string()),
            amount: BigUint::from(ONE_THOUSAND_TOKENS),
            attributes: None,
        };
        self.issue_and_mint_token(third_token_struct, third_token_mint)
            .await;

        let forth_token_struct = IssueTokenStruct {
            token_display_name: "DYN".to_string(),
            token_ticker: "DYN".to_string(),
            token_type: EsdtTokenType::DynamicNFT,
            num_decimals: 10,
        };
        let forth_token_mint = MintTokenStruct {
            name: Some("DYN".to_string()),
            amount: BigUint::from(1u64),
            attributes: None,
        };
        self.issue_and_mint_token(forth_token_struct, forth_token_mint)
            .await;

        let fifth_token_struct = IssueTokenStruct {
            token_display_name: "META".to_string(),
            token_ticker: "META".to_string(),
            token_type: EsdtTokenType::Meta,
            num_decimals: 18,
        };
        let fifth_token_mint = MintTokenStruct {
            name: Some("META".to_string()),
            amount: BigUint::from(ONE_THOUSAND_TOKENS),
            attributes: None,
        };
        self.issue_and_mint_token(fifth_token_struct, fifth_token_mint)
            .await;
    }

    pub async fn deploy_contracts(
        &mut self,
        sovereign_config: OptionalValue<SovereignConfig<StaticApi>>,
        esdt_safe_config: OptionalValue<EsdtSafeConfig<StaticApi>>,
        fee_struct: Option<FeeStruct<StaticApi>>,
    ) {
        self.deploy_chain_config(sovereign_config).await;
        self.deploy_header_verifier(self.state.current_chain_config_sc_address().clone())
            .await;
        self.complete_header_verifier_setup_phase().await;
        self.deploy_mvx_esdt_safe(esdt_safe_config).await;
        self.complete_setup_phase().await;
        self.deploy_fee_market(
            self.state.current_mvx_esdt_safe_contract_address().clone(),
            fee_struct,
        )
        .await;
        self.set_fee_market_address(self.state.current_fee_market_address().to_address())
            .await;
    }

    pub async fn register_operation(
        &mut self,
        signature: ManagedBuffer<StaticApi>,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        operations_hashes: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>>,
    ) {
        self.interactor
            .tx()
            .from(&self.owner_address)
            .to(self.state.current_header_verifier_address())
            .gas(90_000_000u64)
            .typed(HeaderverifierProxy)
            .register_bridge_operations(
                signature,
                hash_of_hashes,
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                operations_hashes,
            )
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
    }

    pub async fn complete_setup_phase(&mut self) {
        self.interactor
            .tx()
            .from(&self.owner_address)
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .complete_setup_phase()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
    }

    pub async fn upgrade(&mut self) {
        let response = self
            .interactor
            .tx()
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .from(&self.owner_address)
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .upgrade()
            .code(MVX_ESDT_SAFE_CODE_PATH)
            .code_metadata(CodeMetadata::UPGRADEABLE)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn update_configuration(&mut self, new_config: EsdtSafeConfig<StaticApi>) {
        let response = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .update_esdt_safe_config_during_setup_phase(new_config)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn set_fee_market_address(&mut self, fee_market_address: Address) {
        let response = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .set_fee_market_address(fee_market_address)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn deposit(
        &mut self,
        to: Address,
        opt_transfer_data: OptionalValueTransferDataTuple<StaticApi>,
        payments: PaymentsVec<StaticApi>,
        expected_error_message: Option<&str>,
        expected_log: Option<&str>,
    ) {
        let (response, logs) = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .deposit(to, opt_transfer_data)
            .payment(payments)
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run()
            .await;

        self.assert_expected_error_message(response, expected_error_message);

        self.assert_expected_log(logs, expected_log);
    }

    pub async fn execute_operations(
        &mut self,
        hash_of_hashes: ManagedBuffer<StaticApi>,
        operation: Operation<StaticApi>,
        expected_error_message: Option<&str>,
        expected_log: Option<&str>,
    ) {
        let (response, logs) = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .gas(120_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .execute_operations(hash_of_hashes, operation)
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run()
            .await;

        self.assert_expected_error_message(response, expected_error_message);

        self.assert_expected_log(logs, expected_log);
    }

    pub async fn register_token(
        &mut self,
        args: RegisterTokenArgs<'_>,
        egld_amount: BigUint<StaticApi>,
        expected_error_message: Option<&str>,
    ) {
        let response = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .register_token(
                args.sov_token_id,
                args.token_type,
                args.token_display_name,
                args.token_ticker,
                args.num_decimals,
            )
            .egld(egld_amount)
            .returns(ReturnsHandledOrError::new())
            .run()
            .await;

        self.assert_expected_error_message(response, expected_error_message);
    }

    pub async fn pause_endpoint(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
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
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
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
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .typed(MvxEsdtSafeProxy)
            .paused_status()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }
}
