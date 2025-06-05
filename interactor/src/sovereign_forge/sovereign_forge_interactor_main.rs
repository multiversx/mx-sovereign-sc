#![allow(non_snake_case)]
use common_interactor::common_sovereign_interactor::{IssueTokenStruct, MintTokenStruct};
use common_interactor::constants::ONE_THOUSAND_TOKENS;
use common_interactor::interactor_state::State;
use common_interactor::{
    common_sovereign_interactor::CommonInteractorTrait, interactor_config::Config,
};
use common_test_setup::constants::SOVEREIGN_FORGE_CODE_PATH;
use multiversx_sc_snippets::imports::*;
use proxies::sovereign_forge_proxy::SovereignForgeProxy;

pub struct SovereignForgeInteract {
    interactor: Interactor,
    alice_address: Address,
    pub state: State,
}
impl CommonInteractorTrait for SovereignForgeInteract {
    fn interactor(&mut self) -> &mut Interactor {
        &mut self.interactor
    }

    fn wallet_address(&self) -> &Address {
        &self.alice_address
    }

    fn state(&mut self) -> &mut State {
        &mut self.state
    }
}
impl SovereignForgeInteract {
    pub async fn new(config: Config) -> Self {
        let mut interactor = Self::initialize_interactor(config).await;
        interactor.initialize_tokens_in_wallets().await;
        interactor
    }

    async fn initialize_interactor(config: Config) -> Self {
        let mut interactor = Interactor::new(config.gateway_uri())
            .await
            .use_chain_simulator(config.use_chain_simulator());

        let current_working_dir = "interactor";
        interactor.set_current_dir_from_workspace(current_working_dir);
        let alice_address = interactor.register_wallet(test_wallets::alice()).await;

        interactor.generate_blocks_until_epoch(1).await.unwrap();

        SovereignForgeInteract {
            interactor,
            alice_address,
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
    }

    pub async fn upgrade(&mut self) {
        let response = self
            .interactor
            .tx()
            .to(self.state.current_sovereign_forge_sc_address())
            .from(&self.alice_address)
            .gas(50_000_000u64)
            .typed(SovereignForgeProxy)
            .upgrade()
            .code(SOVEREIGN_FORGE_CODE_PATH)
            .code_metadata(CodeMetadata::UPGRADEABLE)
            .returns(ReturnsNewAddress)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn register_token_handler(&mut self, shard_id: u32) {
        let address = self.state.current_token_handler_address().to_address();
        let token_handler_address = ManagedAddress::from(address);

        let response = self
            .interactor
            .tx()
            .from(&self.alice_address)
            .to(self.state.current_sovereign_forge_sc_address())
            .gas(30_000_000u64)
            .typed(SovereignForgeProxy)
            .register_token_handler(shard_id, token_handler_address)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn register_chain_factory(&mut self, shard_id: u32) {
        let response = self
            .interactor
            .tx()
            .from(&self.alice_address)
            .to(self.state.current_sovereign_forge_sc_address())
            .gas(30_000_000u64)
            .typed(SovereignForgeProxy)
            .register_chain_factory(shard_id, self.state.current_chain_factory_sc_address())
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn get_chain_factories(&mut self) {
        let shard_id = 0u32;

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_sovereign_forge_sc_address())
            .typed(SovereignForgeProxy)
            .chain_factories(shard_id)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    pub async fn get_token_handlers(&mut self) {
        let shard_id = 0u32;

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_sovereign_forge_sc_address())
            .typed(SovereignForgeProxy)
            .token_handlers(shard_id)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    pub async fn get_deploy_cost(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_sovereign_forge_sc_address())
            .typed(SovereignForgeProxy)
            .deploy_cost()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    pub async fn get_chain_ids(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_sovereign_forge_sc_address())
            .typed(SovereignForgeProxy)
            .chain_ids()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    pub async fn check_setup_phase_status(&mut self, chain_id: &str, expected_value: bool) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_sovereign_forge_sc_address())
            .typed(SovereignForgeProxy)
            .sovereign_setup_phase(chain_id)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        assert_eq!(
            result_value, expected_value,
            "Expected setup phase status to be {expected_value}, but got {result_value}"
        );
    }
}
