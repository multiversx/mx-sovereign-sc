#![allow(non_snake_case)]

use common_interactor::interactor_common_state::CommonState;
use common_interactor::interactor_helpers::InteractorHelpers;
use common_interactor::interactor_state::{EsdtTokenInfo, State};
use common_interactor::interactor_structs::{ActionConfig, BalanceCheckConfig};
use common_interactor::{
    common_sovereign_interactor::CommonInteractorTrait, interactor_config::Config,
};
use common_test_setup::constants::{
    INTERACTOR_WORKING_DIR, ONE_THOUSAND_TOKENS, SOVEREIGN_RECEIVER_ADDRESS, TOKEN_DISPLAY_NAME,
    TOKEN_TICKER,
};
use cross_chain::DEFAULT_ISSUE_COST;
use multiversx_sc::chain_core::EGLD_000000_TOKEN_IDENTIFIER;
use multiversx_sc_snippets::imports::*;
use multiversx_sc_snippets::multiversx_sc_scenario::multiversx_chain_vm::crypto_functions::sha256;
use structs::aliases::PaymentsVec;
use structs::fee::FeeStruct;
use structs::operation::OperationData;
use structs::{OperationHashStatus, RegisterTokenOperation};

pub struct CompleteFlowInteract {
    pub interactor: Interactor,
    pub user_address: Address,
    pub state: State,
    pub common_state: CommonState,
}

impl InteractorHelpers for CompleteFlowInteract {
    fn interactor(&mut self) -> &mut Interactor {
        &mut self.interactor
    }

    fn state(&mut self) -> &mut State {
        &mut self.state
    }

    fn common_state(&mut self) -> &mut CommonState {
        &mut self.common_state
    }

    fn user_address(&self) -> &Address {
        &self.user_address
    }
}
impl CommonInteractorTrait for CompleteFlowInteract {}

impl CompleteFlowInteract {
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

        let current_working_dir = INTERACTOR_WORKING_DIR;
        interactor.set_current_dir_from_workspace(current_working_dir);

        let user_address = interactor.register_wallet(test_wallets::grace()).await;

        interactor.generate_blocks_until_all_activations().await;

        CompleteFlowInteract {
            interactor,
            user_address,
            state: State::default(),
            common_state: CommonState::load_state(),
        }
    }

    async fn initialize_tokens_in_wallets(&mut self) {
        let token_configs = [
            ("MVX", EsdtTokenType::Fungible, 18),
            ("FEE", EsdtTokenType::Fungible, 18),
            ("NFT", EsdtTokenType::NonFungibleV2, 0),
            ("SFT", EsdtTokenType::SemiFungible, 0),
            ("DYN", EsdtTokenType::DynamicNFT, 10),
            ("META", EsdtTokenType::MetaFungible, 0),
            ("DYNS", EsdtTokenType::DynamicSFT, 18),
            ("DYNM", EsdtTokenType::DynamicMeta, 18),
        ];

        for (ticker, token_type, decimals) in token_configs {
            if ticker == "FEE" && !self.common_state.fee_market_tokens.is_empty() {
                let fee_token = self.retrieve_current_fee_token_for_wallet().await;
                self.state.set_fee_token(fee_token);
                continue;
            }
            let amount = match token_type {
                EsdtTokenType::NonFungibleV2 | EsdtTokenType::DynamicNFT => BigUint::from(1u64),
                _ => BigUint::from(ONE_THOUSAND_TOKENS),
            };

            self.create_token_with_config(token_type, ticker, amount, decimals)
                .await;
        }
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
            payment_vec.clone(),
            None,
            expected_log.as_deref(),
        )
        .await;

        let amount = token.as_ref().map(|t| t.amount.clone()).unwrap_or_default();

        let balance_config = BalanceCheckConfig::new()
            .shard(config.shard)
            .token(token.clone())
            .amount(amount)
            .fee(fee.clone())
            .with_transfer_data(config.with_transfer_data.unwrap_or_default())
            .is_execute(false);

        self.check_balances_after_action(balance_config).await;
        self.update_fee_market_balance_state(fee, payment_vec, config.shard)
            .await;
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

        self.register_operation(config.shard, &hash_of_hashes, operations_hashes)
            .await;

        let expected_operation_hash_status = OperationHashStatus::NotLocked;

        self.check_registered_operation_status(
            config.shard,
            &hash_of_hashes,
            operation_hash.clone(),
            expected_operation_hash_status,
        )
        .await;

        let caller = self.get_bridge_service_for_shard(config.shard);
        self.execute_operations_in_mvx_esdt_safe(
            caller,
            config.shard,
            hash_of_hashes.clone(),
            operation.clone(),
            config.expected_error.as_deref(),
            expected_log.as_deref(),
            config.expected_log_error.as_deref(),
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
            RegisterTokenOperation {
                token_id: token.token_id.clone(),
                token_type: token.token_type,
                token_display_name: ManagedBuffer::from(TOKEN_DISPLAY_NAME),
                token_ticker: ManagedBuffer::from(
                    token
                        .token_id
                        .into_managed_buffer()
                        .to_string()
                        .split('-')
                        .nth(1)
                        .unwrap_or(TOKEN_TICKER),
                ),
                num_decimals: token.decimals,
                data: OperationData::new(0u64, self.user_address().into(), None),
            },
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
        self.deposit_in_mvx_esdt_safe(
            SOVEREIGN_RECEIVER_ADDRESS.to_address(),
            config.shard,
            OptionalValue::None,
            ManagedVec::from_single_item(EgldOrEsdtTokenPayment::egld_payment(
                DEFAULT_ISSUE_COST.into(),
            )),
            None,
            Some(EGLD_000000_TOKEN_IDENTIFIER),
        )
        .await;

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

    pub async fn update_fee_market_balance_state(
        &mut self,
        fee: Option<FeeStruct<StaticApi>>,
        payment_vec: PaymentsVec<StaticApi>,
        shard: u32,
    ) {
        if fee.is_none() || payment_vec.is_empty() {
            return;
        }
        let mut fee_token_in_fee_market = self.common_state().get_fee_market_token_for_shard(shard);

        let payment = payment_vec.get(0);
        if let Some(payment_amount) = payment.amount.to_u64() {
            fee_token_in_fee_market.amount += payment_amount;
        }
        self.common_state()
            .set_fee_market_token_for_shard(shard, fee_token_in_fee_market);
    }
}
