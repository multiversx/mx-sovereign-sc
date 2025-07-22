use common_interactor::common_sovereign_interactor::{
    CommonInteractorTrait, IssueTokenStruct, MintTokenStruct,
};
use multiversx_sc_snippets::imports::*;
use proxies::mvx_esdt_safe_proxy::MvxEsdtSafeProxy;

use structs::configs::EsdtSafeConfig;

use common_interactor::interactor_config::Config;
use common_interactor::interactor_state::State;

use common_test_setup::constants::{
    INTERACTOR_WORKING_DIR, MVX_ESDT_SAFE_CODE_PATH, ONE_THOUSAND_TOKENS, SHARD_0,
};

pub struct MvxEsdtSafeInteract {
    pub interactor: Interactor,
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

    fn user_address(&self) -> &Address {
        &self.user_address
    }
}

impl MvxEsdtSafeInteract {
    pub async fn new(config: Config) -> Self {
        let mut interactor = Self::initialize_interactor(config.clone()).await;

        interactor.register_wallets().await;

        match config.use_chain_simulator() {
            true => {
                interactor.initialize_tokens_in_wallets().await;
            }
            false => {
                println!("Skipping token initialization for real network");
            }
        }

        interactor
    }

    async fn initialize_interactor(config: Config) -> Self {
        let mut interactor = Interactor::new(config.gateway_uri())
            .await
            .use_chain_simulator(config.use_chain_simulator());

        let working_dir = INTERACTOR_WORKING_DIR;
        interactor.set_current_dir_from_workspace(working_dir);

        let user_address = interactor.register_wallet(test_wallets::grace()).await; //shard 1

        interactor.generate_blocks_until_epoch(1u64).await.unwrap();

        MvxEsdtSafeInteract {
            interactor,
            user_address,
            state: State::default(),
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

        let initial_balance = vec![
            self.thousand_tokens(self.state.get_first_token_id_string()),
            self.thousand_tokens(self.state.get_second_token_id_string()),
            self.thousand_tokens(self.state.get_fee_token_id_string()),
        ];

        self.state
            .set_initial_balance(self.user_address.to_bech32_default(), initial_balance);
    }

    pub async fn upgrade(&mut self) {
        let caller = self.get_bridge_owner_for_shard(SHARD_0).clone();
        let response = self
            .interactor
            .tx()
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .from(caller)
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

    pub async fn update_configuration(
        &mut self,
        shard: u32,
        hash_of_hashes: ManagedBuffer<StaticApi>,
        new_config: EsdtSafeConfig<StaticApi>,
        expected_error_message: Option<&str>,
        expected_log: Option<&str>,
        expected_log_error: Option<&str>,
    ) {
        let bridge_service = self.get_bridge_service_for_shard(shard);
        let (response, logs) = self
            .interactor
            .tx()
            .from(bridge_service)
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .update_esdt_safe_config(hash_of_hashes, new_config)
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run()
            .await;

        self.assert_expected_error_message(response, expected_error_message);

        self.assert_expected_log(logs, expected_log, expected_log_error);
    }

    pub async fn set_fee_market_address(&mut self, caller: Address, fee_market_address: Address) {
        let response = self
            .interactor
            .tx()
            .from(caller)
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .set_fee_market_address(fee_market_address)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn register_native_token(
        &mut self,
        token_ticker: &str,
        token_name: &str,
        egld_amount: BigUint<StaticApi>,
        expected_error_message: Option<&str>,
    ) {
        let caller = self.get_bridge_owner_for_shard(SHARD_0).clone();
        let response = self
            .interactor
            .tx()
            .from(&caller)
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .register_native_token(token_ticker, token_name)
            .egld(egld_amount)
            .returns(ReturnsHandledOrError::new())
            .run()
            .await;

        self.assert_expected_error_message(response, expected_error_message);
    }
}
