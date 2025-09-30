use common_interactor::{
    common_sovereign_interactor::CommonInteractorTrait, interactor_common_state::CommonState,
    interactor_helpers::InteractorHelpers,
};
use multiversx_sc_snippets::{
    imports::*, multiversx_sc_scenario::multiversx_chain_vm::crypto_functions::sha256,
};
use proxies::mvx_esdt_safe_proxy::MvxEsdtSafeProxy;

use structs::{configs::EsdtSafeConfig, generate_hash::GenerateHash};

use common_interactor::interactor_config::Config;
use common_interactor::interactor_state::State;

use common_test_setup::constants::{
    INTERACTOR_WORKING_DIR, MVX_ESDT_SAFE_CODE_PATH, ONE_THOUSAND_TOKENS, SHARD_0,
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
        ];

        for (ticker, token_type, decimals) in token_configs {
            let amount = if matches!(
                token_type,
                EsdtTokenType::NonFungibleV2 | EsdtTokenType::DynamicNFT
            ) {
                BigUint::from(1u64)
            } else {
                BigUint::from(ONE_THOUSAND_TOKENS)
            };

            self.create_token_with_config(token_type, ticker, amount, decimals)
                .await;
        }
    }

    pub async fn complete_setup_phase(&mut self, shard: u32) {
        let caller = self.get_bridge_owner_for_shard(shard).clone();
        self.interactor
            .tx()
            .from(&caller)
            .to(self.common_state.current_mvx_esdt_safe_contract_address())
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .complete_setup_phase()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
    }

    pub async fn upgrade(&mut self) {
        let caller = self.get_bridge_owner_for_shard(SHARD_0).clone();
        let response = self
            .interactor
            .tx()
            .to(self.common_state.current_mvx_esdt_safe_contract_address())
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

    pub async fn update_configuration_after_setup_phase(
        &mut self,
        shard: u32,
        config: EsdtSafeConfig<StaticApi>,
        expected_log: Option<&str>,
        expected_log_error: Option<&str>,
    ) {
        let bridge_service = self.get_bridge_service_for_shard(shard);
        let config_hash = config.generate_hash();

        self.common_state().update_config_nonce += 1;
        let nonce_str = self.common_state().update_config_nonce.to_string();
        let nonce_buf = ManagedBuffer::<StaticApi>::from(&nonce_str);

        let mut bytes = Vec::with_capacity(config_hash.len() + nonce_buf.len());
        bytes.extend_from_slice(&config_hash.to_vec());
        bytes.extend_from_slice(&nonce_buf.to_vec());

        let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&bytes));
        let operations_hashes =
            MultiValueEncoded::from(ManagedVec::from(vec![config_hash.clone(), nonce_buf]));

        self.register_operation(shard, &hash_of_hashes, operations_hashes)
            .await;

        let (response, logs) = self
            .interactor
            .tx()
            .from(bridge_service)
            .to(self.common_state.current_mvx_esdt_safe_contract_address())
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .update_esdt_safe_config(hash_of_hashes, config)
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run()
            .await;

        self.assert_expected_error_message(response, None);

        self.assert_expected_log(logs, expected_log, expected_log_error);
    }

    pub async fn set_fee_market_address(&mut self, caller: Address, fee_market_address: Address) {
        let response = self
            .interactor
            .tx()
            .from(caller)
            .to(self.common_state.current_mvx_esdt_safe_contract_address())
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .set_fee_market_address(fee_market_address)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }
}
