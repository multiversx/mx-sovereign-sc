#![allow(non_snake_case)]
use common_interactor::common_sovereign_interactor::{IssueTokenStruct, MintTokenStruct};
use common_interactor::interactor_state::State;
use common_interactor::{
    common_sovereign_interactor::CommonInteractorTrait, interactor_config::Config,
};
use common_test_setup::constants::{
    INTERACTOR_WORKING_DIR, ONE_THOUSAND_TOKENS, SOVEREIGN_FORGE_CODE_PATH,
};
use multiversx_sc_snippets::imports::*;
use proxies::sovereign_forge_proxy::SovereignForgeProxy;
use structs::configs::{EsdtSafeConfig, SovereignConfig};
use structs::fee::FeeStruct;
use structs::forge::ScArray;

pub struct SovereignForgeInteract {
    pub interactor: Interactor,
    pub bridge_owner: Address,
    pub sovereign_owner: Address,
    pub bridge_service: Address,
    pub user_address: Address,
    pub state: State,
}
impl CommonInteractorTrait for SovereignForgeInteract {
    fn interactor(&mut self) -> &mut Interactor {
        &mut self.interactor
    }

    fn bridge_owner(&self) -> &Address {
        &self.bridge_owner
    }

    fn sovereign_owner(&self) -> &Address {
        &self.sovereign_owner
    }

    fn bridge_service(&self) -> &Address {
        &self.bridge_service
    }

    fn user_address(&self) -> &Address {
        &self.user_address
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

        let current_working_dir = INTERACTOR_WORKING_DIR;
        interactor.set_current_dir_from_workspace(current_working_dir);
        let bridge_owner = interactor.register_wallet(test_wallets::mike()).await;
        let sovereign_owner = interactor.register_wallet(test_wallets::alice()).await;
        let bridge_service = interactor.register_wallet(test_wallets::carol()).await;
        let user_address = interactor.register_wallet(test_wallets::bob()).await;

        interactor.generate_blocks_until_epoch(1).await.unwrap();

        SovereignForgeInteract {
            interactor,
            bridge_owner,
            sovereign_owner,
            bridge_service,
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

        let fee_token_struct = IssueTokenStruct {
            token_display_name: "FEE".to_string(),
            token_ticker: "FEE".to_string(),
            token_type: EsdtTokenType::Fungible,
            num_decimals: 18,
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
    }

    pub async fn deploy_template_contracts(&mut self) {
        self.deploy_chain_config(OptionalValue::None).await;

        self.deploy_mvx_esdt_safe(OptionalValue::None).await;

        self.deploy_fee_market(
            self.state.current_mvx_esdt_safe_contract_address().clone(),
            None,
        )
        .await;

        self.deploy_header_verifier(Vec::new()).await;
    }

    pub async fn deploy_and_complete_setup_phase(
        &mut self,
        chain_id: &str,
        deploy_cost: BigUint<StaticApi>,
        optional_sov_config: OptionalValue<SovereignConfig<StaticApi>>,
        optional_esdt_safe_config: OptionalValue<EsdtSafeConfig<StaticApi>>,
        fee: Option<FeeStruct<StaticApi>>,
    ) {
        self.deploy_template_contracts().await;
        self.deploy_sovereign_forge(&deploy_cost).await;

        let sov_forge_address = self.state.current_sovereign_forge_sc_address().clone();
        let chain_config_address = self.state.current_chain_config_sc_address().clone();
        let mvx_esdt_safe_address = self.state.current_mvx_esdt_safe_contract_address().clone();
        let fee_market_address = self.state.current_fee_market_address().clone();
        let header_verifier_address = self.state.current_header_verifier_address().clone();

        self.deploy_chain_factory(
            sov_forge_address,
            chain_config_address.clone(),
            header_verifier_address,
            mvx_esdt_safe_address,
            fee_market_address,
        )
        .await;
        let chain_factory_address = self.state().current_chain_factory_sc_address().clone();

        self.deploy_token_handler(chain_factory_address.to_address())
            .await;

        self.register_token_handler(0).await;
        self.register_token_handler(1).await;
        self.register_token_handler(2).await;
        self.register_token_handler(3).await;
        self.register_chain_factory(0).await;
        self.register_chain_factory(1).await;
        self.register_chain_factory(2).await;
        self.register_chain_factory(3).await;

        self.deploy_phase_one(deploy_cost, Some(chain_id.into()), optional_sov_config)
            .await;

        self.register_as_validator(
            ManagedBuffer::from("genesis_validator"),
            MultiEgldOrEsdtPayment::new(),
            chain_config_address,
        )
        .await;

        self.deploy_phase_two(optional_esdt_safe_config).await;
        self.deploy_phase_three(fee).await;
        self.deploy_phase_four().await;

        self.complete_setup_phase().await;
        self.check_setup_phase_status(chain_id, true).await;
        self.update_smart_contracts_addresses_in_state(chain_id)
            .await;
    }

    pub async fn upgrade(&mut self) {
        let response = self
            .interactor
            .tx()
            .to(self.state.current_sovereign_forge_sc_address())
            .from(self.bridge_owner.clone())
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
            .from(&self.bridge_owner.clone())
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
            .from(&self.bridge_owner.clone())
            .to(self.state.current_sovereign_forge_sc_address())
            .gas(30_000_000u64)
            .typed(SovereignForgeProxy)
            .register_chain_factory(shard_id, self.state.current_chain_factory_sc_address())
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn update_smart_contracts_addresses_in_state(&mut self, chain_id: &str) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_sovereign_forge_sc_address())
            .typed(SovereignForgeProxy)
            .sovereign_deployed_contracts(chain_id)
            .returns(ReturnsResult)
            .run()
            .await;

        for contract in result_value {
            match contract.id {
                ScArray::ChainConfig => {
                    self.state.set_chain_factory_sc_address(Bech32Address::from(
                        contract.address.to_address(),
                    ));
                }
                ScArray::ESDTSafe => {
                    self.state
                        .set_mvx_esdt_safe_contract_address(Bech32Address::from(
                            contract.address.to_address(),
                        ));
                }
                ScArray::FeeMarket => {
                    self.state
                        .set_fee_market_address(Bech32Address::from(contract.address.to_address()));
                }
                ScArray::HeaderVerifier => {
                    self.state.set_header_verifier_address(Bech32Address::from(
                        contract.address.to_address(),
                    ));
                }
                _ => {}
            }
        }
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
