use common_test_setup::{
    constants::{
        CHAIN_FACTORY_SC_ADDRESS, OWNER_ADDRESS, OWNER_BALANCE, SOVEREIGN_FORGE_SC_ADDRESS,
    },
    AccountSetup, BaseSetup,
};
use multiversx_sc::imports::OptionalValue;
use multiversx_sc_scenario::{api::StaticApi, ReturnsHandledOrError, ScenarioTxRun};
use proxies::chain_factory_proxy::ChainFactoryContractProxy;
use structs::configs::SovereignConfig;

pub struct ChainFactoryTestState {
    pub common_setup: BaseSetup,
}

impl ChainFactoryTestState {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let owner_account = AccountSetup {
            address: OWNER_ADDRESS.to_address(),
            code_path: None,
            egld_balance: Some(OWNER_BALANCE.into()),
            esdt_balances: None,
        };

        let account_setups = vec![owner_account];

        let common_setup = BaseSetup::new(account_setups);

        Self { common_setup }
    }

    pub fn deploy_chain_config_from_factory(
        &mut self,
        opt_config: OptionalValue<SovereignConfig<StaticApi>>,
        error_message: Option<&str>,
    ) {
        let response = self
            .common_setup
            .world
            .tx()
            .from(SOVEREIGN_FORGE_SC_ADDRESS)
            .to(CHAIN_FACTORY_SC_ADDRESS)
            .typed(ChainFactoryContractProxy)
            .deploy_sovereign_chain_config_contract(opt_config)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(response, error_message);
    }
}
