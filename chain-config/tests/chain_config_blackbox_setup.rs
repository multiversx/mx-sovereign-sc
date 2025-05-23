use common_test_setup::{
    constants::{CHAIN_CONFIG_ADDRESS, OWNER_ADDRESS, OWNER_BALANCE},
    AccountSetup, BaseSetup,
};
use multiversx_sc_scenario::{api::StaticApi, ReturnsHandledOrError, ScenarioTxRun};
use proxies::chain_config_proxy::ChainConfigContractProxy;
use structs::configs::SovereignConfig;

pub struct ChainConfigTestState {
    pub common_setup: BaseSetup,
}

impl ChainConfigTestState {
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

    pub fn update_chain_config(
        &mut self,
        config: SovereignConfig<StaticApi>,
        expect_error: Option<&str>,
    ) {
        let transaction = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(CHAIN_CONFIG_ADDRESS)
            .typed(ChainConfigContractProxy)
            .update_config(config)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(transaction, expect_error);
    }
}
