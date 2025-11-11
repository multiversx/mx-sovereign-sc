use common_interactor::{
    common_sovereign_interactor::CommonInteractorTrait, interactor_common_state::CommonState,
    interactor_helpers::InteractorHelpers,
};
use common_test_setup::base_setup::init::ExpectedLogs;
use multiversx_sc_snippets::{
    imports::*, multiversx_sc_scenario::multiversx_chain_vm::crypto_functions::sha256,
};
use proxies::mvx_esdt_safe_proxy::MvxEsdtSafeProxy;

use structs::{
    configs::{EsdtSafeConfig, UpdateEsdtSafeConfigOperation},
    generate_hash::GenerateHash,
};

use common_interactor::interactor_config::Config;
use common_interactor::interactor_state::State;

use common_test_setup::{
    base_setup::log_validations::assert_expected_logs,
    constants::{
        EXECUTED_BRIDGE_OP_EVENT, INTERACTOR_WORKING_DIR, MVX_ESDT_SAFE_CODE_PATH, SHARD_0,
        UPDATE_ESDT_SAFE_CONFIG_ENDPOINT,
    },
    log,
};

pub struct MvxEsdtSafeInteract {
    pub interactor: Interactor,
    pub user_address: Address,
    pub state: State,
    pub common_state: CommonState,
}

impl InteractorHelpers for MvxEsdtSafeInteract {
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
impl CommonInteractorTrait for MvxEsdtSafeInteract {}

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

        interactor.generate_blocks_until_all_activations().await;

        MvxEsdtSafeInteract {
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
            ("TRUSTED", EsdtTokenType::Fungible, 18),
        ];

        for (ticker, token_type, decimals) in token_configs {
            self.create_token_with_config(token_type, ticker, decimals)
                .await;
        }
    }

    pub async fn complete_setup_phase(&mut self, shard: u32) {
        let caller = self.get_bridge_owner_for_shard(shard).clone();
        self.interactor
            .tx()
            .from(&caller)
            .to(self.common_state.get_mvx_esdt_safe_address(shard).clone())
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .complete_setup_phase()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
    }

    pub async fn upgrade(&mut self) {
        let caller = self.get_bridge_owner_for_shard(SHARD_0).clone();
        self.interactor
            .tx()
            .to(self.common_state.get_mvx_esdt_safe_address(SHARD_0).clone())
            .from(caller)
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .upgrade()
            .code(MVX_ESDT_SAFE_CODE_PATH)
            .code_metadata(CodeMetadata::UPGRADEABLE)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
    }

    pub async fn update_configuration_after_setup_phase(
        &mut self,
        shard: u32,
        esdt_safe_config: EsdtSafeConfig<StaticApi>,
        expected_log_error: Option<&str>,
    ) {
        let bridge_service = self.get_bridge_service_for_shard(shard);
        let mvx_esdt_safe_address = self.common_state.get_mvx_esdt_safe_address(shard).clone();

        let operation: UpdateEsdtSafeConfigOperation<StaticApi> = UpdateEsdtSafeConfigOperation {
            esdt_safe_config,
            nonce: self
                .common_state()
                .get_and_increment_operation_nonce(&mvx_esdt_safe_address.to_string()),
        };

        let operation_hash = operation.generate_hash();
        let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

        let operations_hashes = MultiValueEncoded::from_iter(vec![operation_hash.clone()]);

        self.register_operation(shard, &hash_of_hashes, operations_hashes)
            .await;

        let (response, logs) = self
            .interactor
            .tx()
            .from(bridge_service)
            .to(mvx_esdt_safe_address)
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .update_esdt_safe_config(hash_of_hashes, operation)
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run()
            .await;

        self.assert_expected_error_message(response, None);

        let expected_logs = if expected_log_error.is_some() {
            vec![
                log!(UPDATE_ESDT_SAFE_CONFIG_ENDPOINT, topics: [EXECUTED_BRIDGE_OP_EVENT], data: expected_log_error),
            ]
        } else {
            vec![log!(UPDATE_ESDT_SAFE_CONFIG_ENDPOINT, topics: [EXECUTED_BRIDGE_OP_EVENT])]
        };
        assert_expected_logs(logs, expected_logs);
    }

    pub async fn set_fee_market_address(&mut self, caller: Address, fee_market_address: Address) {
        self.interactor
            .tx()
            .from(caller)
            .to(self.common_state.get_mvx_esdt_safe_address(SHARD_0).clone())
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .set_fee_market_address(fee_market_address)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
    }

    pub async fn check_deposited_tokens_amount(
        &mut self,
        token_id: EgldOrEsdtTokenIdentifier<StaticApi>,
        shard: u32,
        expected_amount: BigUint<StaticApi>,
    ) {
        let mvx_esdt_safe_address = self.common_state.get_mvx_esdt_safe_address(shard).clone();
        let result = self
            .interactor()
            .query()
            .to(mvx_esdt_safe_address)
            .typed(MvxEsdtSafeProxy)
            .deposited_tokens_amount(token_id)
            .returns(ReturnsResult)
            .run()
            .await;

        assert_eq!(result, expected_amount, "Incorrect deposited tokens amount");
    }
}
