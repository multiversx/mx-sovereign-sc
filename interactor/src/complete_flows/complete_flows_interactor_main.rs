#![allow(non_snake_case)]
use std::path::Path;

use common_interactor::interactor_helpers::InteractorHelpers;
use common_interactor::interactor_state::{EsdtTokenInfo, State};
use common_interactor::interactor_structs::{ActionConfig, BalanceCheckConfig};
use common_interactor::{
    common_sovereign_interactor::CommonInteractorTrait, interactor_config::Config,
};
use common_test_setup::base_setup::init::RegisterTokenArgs;
use common_test_setup::constants::{
    INTERACTOR_WORKING_DIR, ISSUE_COST, ONE_THOUSAND_TOKENS, OPERATION_HASH_STATUS_STORAGE_KEY,
    SOVEREIGN_RECEIVER_ADDRESS, TOKEN_DISPLAY_NAME, TOKEN_TICKER,
};
use header_verifier::utils::OperationHashStatus;
use multiversx_sc_snippets::multiversx_sc_scenario::multiversx_chain_vm::crypto_functions::sha256;
use multiversx_sc_snippets::{hex, imports::*};
use structs::fee::FeeStruct;

pub struct CompleteFlowInteract {
    pub interactor: Interactor,
    pub user_address: Address,
    pub state: State,
}

impl InteractorHelpers for CompleteFlowInteract {
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
impl CommonInteractorTrait for CompleteFlowInteract {}

impl CompleteFlowInteract {
    pub async fn new(config: Config) -> Self {
        let mut interactor = Self::initialize_interactor(config.clone()).await;

        interactor.register_wallets(config.test_id).await;

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

        let current_working_dir = INTERACTOR_WORKING_DIR;
        interactor.set_current_dir_from_workspace(current_working_dir);

        let wallets_base_path = "wallets";
        let test_folder = format!("test_{}", config.test_id);
        let test_path = Path::new(wallets_base_path).join(&test_folder);
        let user_wallet_path = test_path.join("user.pem");
        let user_wallet = Self::load_wallet(&user_wallet_path, config.test_id);
        let user_address = interactor.register_wallet(user_wallet).await;

        interactor.generate_blocks_until_epoch(1).await.unwrap();

        CompleteFlowInteract {
            interactor,
            user_address,
            state: State::default(),
        }
    }

    async fn initialize_tokens_in_wallets(&mut self) {
        let token_configs = [
            ("MVX", EsdtTokenType::Fungible, 18),
            ("MVX2", EsdtTokenType::Fungible, 18),
            ("FEE", EsdtTokenType::Fungible, 18),
            ("NFT", EsdtTokenType::NonFungibleV2, 0),
            ("SFT", EsdtTokenType::SemiFungible, 0),
            ("DYN", EsdtTokenType::DynamicNFT, 10),
            ("META", EsdtTokenType::MetaFungible, 0),
            ("DYNS", EsdtTokenType::DynamicSFT, 18),
            ("DYNM", EsdtTokenType::DynamicMeta, 18),
        ];

        let mut all_tokens = Vec::new();

        for (ticker, token_type, decimals) in token_configs {
            let amount = match token_type {
                EsdtTokenType::NonFungibleV2 | EsdtTokenType::DynamicNFT => BigUint::from(1u64),
                _ => BigUint::from(ONE_THOUSAND_TOKENS),
            };

            let token = self
                .create_token_with_config(token_type, ticker, amount, decimals)
                .await;

            match ticker {
                "MVX" => self.state.set_first_token(token.clone()),
                "MVX2" => self.state.set_second_token(token.clone()),
                "FEE" => self.state.set_fee_token(token.clone()),
                "NFT" => self.state.set_nft_token_id(token.clone()),
                "SFT" => self.state.set_sft_token_id(token.clone()),
                "DYN" => self.state.set_dynamic_nft_token_id(token.clone()),
                "META" => self.state.set_meta_esdt_token_id(token.clone()),
                "DYNS" => self.state.set_dynamic_sft_token_id(token.clone()),
                "DYNM" => self.state.set_dynamic_meta_esdt_token_id(token.clone()),
                _ => {}
            }

            all_tokens.push(token);
        }

        self.state.set_initial_wallet_balance(all_tokens);
    }

    pub async fn deposit_wrapper(
        &mut self,
        config: ActionConfig,
        token: Option<EsdtTokenInfo>,
        fee: Option<FeeStruct<StaticApi>>,
    ) {
        let expected_log = self.extract_log_based_on_shard(&config);
        let payment_vec = self.prepare_deposit_payments(
            token.clone(),
            fee.clone(),
            config.with_transfer_data.unwrap_or_default(),
        );

        let transfer_data =
            self.prepare_transfer_data(config.with_transfer_data.unwrap_or_default());

        self.deposit_in_mvx_esdt_safe(
            SOVEREIGN_RECEIVER_ADDRESS.to_address(),
            config.shard,
            transfer_data,
            payment_vec,
            None,
            expected_log.as_deref(),
        )
        .await;

        let amount = token.as_ref().map(|t| t.amount.clone()).unwrap_or_default();

        let balance_config = BalanceCheckConfig::new()
            .shard(config.shard)
            .token(token.clone())
            .amount(amount)
            .fee(fee)
            .with_transfer_data(config.with_transfer_data.unwrap_or_default())
            .is_execute(false);

        self.check_balances_after_action(balance_config).await;
    }

    async fn register_and_execute_operation(
        &mut self,
        config: ActionConfig,
        token: Option<EsdtTokenInfo>,
    ) {
        let expected_log = self.extract_log_based_on_shard(&config);
        let operation = self
            .prepare_operation(token, config.endpoint.as_deref())
            .await;

        let operation_hash = self.get_operation_hash(&operation);
        let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

        let operations_hashes =
            MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));

        self.register_operation(
            config.shard,
            ManagedBuffer::new(),
            &hash_of_hashes,
            operations_hashes,
        )
        .await;

        let operation_status = OperationHashStatus::NotLocked as u8;
        let expected_operation_hash_status = format!("{:02x}", operation_status);
        let encoded_key = &hex::encode(OPERATION_HASH_STATUS_STORAGE_KEY);

        self.check_account_storage(
            self.state
                .get_header_verifier_address(config.shard)
                .to_address(),
            encoded_key,
            Some(&expected_operation_hash_status),
        )
        .await;

        let caller = self.get_bridge_service_for_shard(config.shard);
        self.execute_operations_in_mvx_esdt_safe(
            caller,
            config.shard,
            hash_of_hashes,
            operation,
            config.expected_error.as_deref(),
            expected_log.as_deref(),
            config.expected_log_error.as_deref(),
        )
        .await;

        self.check_account_storage(
            self.state
                .get_header_verifier_address(config.shard)
                .to_address(),
            encoded_key,
            None,
        )
        .await;
    }

    pub async fn execute_wrapper(
        &mut self,
        config: ActionConfig,
        token: Option<EsdtTokenInfo>,
    ) -> Option<EsdtTokenInfo> {
        self.register_and_execute_operation(config.clone(), token.clone())
            .await;

        let (expected_token, expected_amount) = match &token {
            Some(t) if self.is_sovereign_token(t) => {
                let mapped_token = self.get_mapped_token(config.clone(), t, &t.amount).await;
                (Some(mapped_token.clone()), mapped_token.amount)
            }
            _ => {
                let amount = token.as_ref().map(|t| t.amount.clone()).unwrap_or_default();
                (token.clone(), amount)
            }
        };

        let balance_config = BalanceCheckConfig::new()
            .shard(config.shard)
            .token(expected_token.clone())
            .amount(expected_amount)
            .is_execute(true)
            .with_transfer_data(config.with_transfer_data.unwrap_or_default())
            .expected_error(config.expected_error.clone());

        self.check_balances_after_action(balance_config).await;

        expected_token
    }

    async fn register_sovereign_token(&mut self, shard: u32, token: EsdtTokenInfo) -> String {
        self.register_token(
            shard,
            RegisterTokenArgs {
                sov_token_id: TokenIdentifier::from_esdt_bytes(&token.token_id),
                token_type: token.token_type,
                token_display_name: TOKEN_DISPLAY_NAME,
                token_ticker: token.token_id.split('-').nth(1).unwrap_or(TOKEN_TICKER),
                num_decimals: token.decimals,
            },
            ISSUE_COST.into(),
            None,
        )
        .await
        .expect("Failed to register sovereign token")
    }

    pub async fn register_and_execute_sovereign_token(
        &mut self,
        mut config: ActionConfig,
        token: EsdtTokenInfo,
    ) -> EsdtTokenInfo {
        let expected_log = self
            .register_sovereign_token(config.shard, token.clone())
            .await;

        if config.expected_error.is_none() {
            config = config.expect_log(vec![expected_log]);
        }

        self.execute_wrapper(config, Some(token.clone()))
            .await
            .expect("Expected mapped token, got None")
    }
}
