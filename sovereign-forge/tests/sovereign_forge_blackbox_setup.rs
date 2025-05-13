use common_test_setup::{
    constants::{
        CHAIN_FACTORY_SC_ADDRESS, OWNER_ADDRESS, OWNER_BALANCE, SOVEREIGN_FORGE_SC_ADDRESS,
        TOKEN_HANDLER_SC_ADDRESS,
    },
    AccountSetup, BaseSetup,
};
use multiversx_sc::types::{BigUint, ManagedAddress, ReturnsResultUnmanaged, TestSCAddress};
use multiversx_sc_scenario::{api::StaticApi, ReturnsHandledOrError, ScenarioTxRun};
use proxies::sovereign_forge_proxy::{ScArray, SovereignForgeProxy};
use sovereign_forge::common::storage::ChainId;
use structs::configs::{EsdtSafeConfig, SovereignConfig};

pub struct SovereignForgeTestState {
    pub common_setup: BaseSetup,
}

impl SovereignForgeTestState {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let owner_setup = AccountSetup {
            address: OWNER_ADDRESS.to_address(),
            code_path: None,
            esdt_balances: None,
            egld_balance: Some(BigUint::from(OWNER_BALANCE)),
        };

        let account_setups = vec![owner_setup];

        let common_setup = BaseSetup::new(account_setups);

        Self { common_setup }
    }

    pub fn finish_setup(&mut self) {
        self.register_chain_factory(1, CHAIN_FACTORY_SC_ADDRESS, None);
        self.register_chain_factory(2, CHAIN_FACTORY_SC_ADDRESS, None);
        self.register_chain_factory(3, CHAIN_FACTORY_SC_ADDRESS, None);
        self.register_token_handler(1, TOKEN_HANDLER_SC_ADDRESS, None);
        self.register_token_handler(2, TOKEN_HANDLER_SC_ADDRESS, None);
        self.register_token_handler(3, TOKEN_HANDLER_SC_ADDRESS, None);
    }

    pub fn register_token_handler(
        &mut self,
        shard_id: u32,
        token_handler_address: TestSCAddress,
        error_message: Option<&str>,
    ) {
        let response = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(SOVEREIGN_FORGE_SC_ADDRESS)
            .typed(SovereignForgeProxy)
            .register_token_handler(shard_id, token_handler_address)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(response, error_message);
    }

    pub fn register_chain_factory(
        &mut self,
        shard_id: u32,
        chain_factory_address: TestSCAddress,
        error_message: Option<&str>,
    ) {
        let response = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(SOVEREIGN_FORGE_SC_ADDRESS)
            .typed(SovereignForgeProxy)
            .register_chain_factory(shard_id, chain_factory_address)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(response, error_message);
    }

    pub fn update_sovereign_config(
        &mut self,
        new_sovereign_config: SovereignConfig<StaticApi>,
        expected_error_message: Option<&str>,
    ) {
        let response = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(SOVEREIGN_FORGE_SC_ADDRESS)
            .typed(SovereignForgeProxy)
            .update_sovereign_config(new_sovereign_config)
            .returns(ReturnsHandledOrError::new())
            .run();

        if let Err(error) = response {
            assert_eq!(expected_error_message, Some(error.message.as_str()))
        }
    }

    pub fn update_esdt_safe_config(
        &mut self,
        new_esdt_safe_config: EsdtSafeConfig<StaticApi>,
        expected_error_message: Option<&str>,
    ) {
        let response = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(SOVEREIGN_FORGE_SC_ADDRESS)
            .typed(SovereignForgeProxy)
            .update_esdt_safe_config(new_esdt_safe_config)
            .returns(ReturnsHandledOrError::new())
            .run();

        if let Err(error) = response {
            assert_eq!(expected_error_message, Some(error.message.as_str()))
        }
    }

    pub fn complete_setup_phase(&mut self, error_message: Option<&str>) {
        let response = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(SOVEREIGN_FORGE_SC_ADDRESS)
            .typed(SovereignForgeProxy)
            .complete_setup_phase()
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(response, error_message);
    }

    pub fn get_smart_contract_address_from_sovereign_forge(
        &mut self,
        chain_id: ChainId<StaticApi>,
        sc_id: ScArray,
    ) -> ManagedAddress<StaticApi> {
        self.common_setup
            .world
            .query()
            .to(SOVEREIGN_FORGE_SC_ADDRESS)
            .typed(SovereignForgeProxy)
            .sovereign_deployed_contracts(chain_id)
            .returns(ReturnsResultUnmanaged)
            .run()
            .iter()
            .find(|sc| sc.id == sc_id)
            .unwrap()
            .address
            .clone()
    }
}
