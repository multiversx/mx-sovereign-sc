#![allow(non_snake_case)]

use common_interactor::interactor_common_state::CommonState;
use common_interactor::interactor_helpers::InteractorHelpers;
use common_interactor::interactor_state::{EsdtTokenInfo, State};
use common_interactor::interactor_structs::{ActionConfig, BalanceCheckConfig};
use common_interactor::{
    common_sovereign_interactor::CommonInteractorTrait, interactor_config::Config,
};
use common_test_setup::base_setup::init::ExpectedLogs;
use common_test_setup::constants::{
    INTERACTOR_WORKING_DIR, MULTI_ESDT_NFT_TRANSFER_EVENT, SHARD_1, SOVEREIGN_RECEIVER_ADDRESS,
    TOKEN_DISPLAY_NAME, TOKEN_TICKER,
};
use common_test_setup::log;
use cross_chain::DEFAULT_ISSUE_COST;
use error_messages::EXPECTED_MAPPED_TOKEN;
use multiversx_sc::api::{ESDT_LOCAL_MINT_FUNC_NAME, ESDT_NFT_CREATE_FUNC_NAME};
use multiversx_sc::chain_core::EGLD_000000_TOKEN_IDENTIFIER;
use multiversx_sc_snippets::imports::*;
use multiversx_sc_snippets::multiversx_sc_scenario::multiversx_chain_vm::crypto_functions::sha256;
use structs::fee::FeeStruct;
use structs::generate_hash::GenerateHash;
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
            ("TRUSTED", EsdtTokenType::Fungible, 18),
        ];

        for (ticker, token_type, decimals) in token_configs {
            self.create_token_with_config(token_type, ticker, decimals)
                .await;
        }
    }

    pub async fn deposit_wrapper(
        &mut self,
        config: ActionConfig,
        token: Option<EsdtTokenInfo>,
        fee: Option<FeeStruct<StaticApi>>,
    ) {
        let expected_log = self.build_expected_deposit_log(config.clone(), token.clone());
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
            Some(expected_log),
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
        let expected_logs = self.build_expected_execute_log(config.clone(), token.clone());
        let operation = self
            .prepare_operation(config.shard, token, config.endpoint.as_deref())
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
            None,
            Some(expected_logs),
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
            .expected_error(config.expected_log_error);

        self.check_balances_after_action(balance_config).await;

        expected_token
    }

    async fn register_sovereign_token(&mut self, shard: u32, token: EsdtTokenInfo) -> String {
        let mvx_esdt_safe_address = self.common_state().get_mvx_esdt_safe_address(shard).clone();
        let nonce = self
            .common_state()
            .get_and_increment_operation_nonce(&mvx_esdt_safe_address.to_string());

        let operation = RegisterTokenOperation {
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
            data: OperationData::new(nonce, self.user_address().into(), None),
        };

        let operation_hash = operation.generate_hash();
        let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

        let operations_hashes =
            MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));

        self.register_operation(shard, &hash_of_hashes, operations_hashes)
            .await;

        self.register_token(shard, operation).await
    }

    pub async fn register_and_execute_sovereign_token(
        &mut self,
        mut config: ActionConfig,
        token: EsdtTokenInfo,
    ) -> EsdtTokenInfo {
        let expected_deposit_log = match config.shard {
            SHARD_1 => {
                vec![log!(MULTI_ESDT_NFT_TRANSFER_EVENT, topics: [EGLD_000000_TOKEN_IDENTIFIER])]
            }
            _ => vec![],
        };
        self.deposit_in_mvx_esdt_safe(
            SOVEREIGN_RECEIVER_ADDRESS.to_address(),
            config.shard,
            OptionalValue::None,
            ManagedVec::from_single_item(EgldOrEsdtTokenPayment::egld_payment(
                DEFAULT_ISSUE_COST.into(),
            )),
            None,
            Some(expected_deposit_log),
        )
        .await;

        let token_id = self
            .register_sovereign_token(config.shard, token.clone())
            .await;

        let additional_log = if token.token_type != EsdtTokenType::Fungible {
            log!(ESDT_NFT_CREATE_FUNC_NAME, topics: [token_id.clone()])
        } else {
            log!(ESDT_LOCAL_MINT_FUNC_NAME, topics: [token_id])
        };

        config = config.expect_additional_log(additional_log);

        self.execute_wrapper(config, Some(token.clone()))
            .await
            .expect(EXPECTED_MAPPED_TOKEN)
    }
}
