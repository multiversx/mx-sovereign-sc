use common_test_setup::{
    constants::{CHAIN_CONFIG_ADDRESS, OWNER_ADDRESS, OWNER_BALANCE},
    AccountSetup, BaseSetup,
};
use multiversx_sc::types::ManagedBuffer;
use multiversx_sc_scenario::{api::StaticApi, ReturnsHandledOrError, ReturnsLogs, ScenarioTxRun};
use proxies::chain_config_proxy::ChainConfigContractProxy;
use structs::{configs::SovereignConfig, ValidatorInfo};

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

    pub fn update_sovereign_config_during_setup_phase(
        &mut self,
        config: SovereignConfig<StaticApi>,
        expect_error: Option<&str>,
    ) {
        let result = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(CHAIN_CONFIG_ADDRESS)
            .typed(ChainConfigContractProxy)
            .update_sovereign_config_during_setup_phase(config)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(result, expect_error);
    }

    pub fn update_sovereign_config(
        &mut self,
        hash_of_hashes: ManagedBuffer<StaticApi>,
        config: SovereignConfig<StaticApi>,
        expect_error: Option<&str>,
        expected_custom_log: Option<&str>,
    ) {
        let (result, logs) = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(CHAIN_CONFIG_ADDRESS)
            .typed(ChainConfigContractProxy)
            .update_sovereign_config(hash_of_hashes, config)
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run();

        self.common_setup
            .assert_expected_error_message(result, expect_error);

        self.common_setup
            .assert_expected_log(logs, expected_custom_log);
    }

    pub fn register(
        &mut self,
        new_validator: &ValidatorInfo<StaticApi>,
        expect_error: Option<&str>,
        expected_custom_log: Option<&str>,
    ) {
        let (result, logs) = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(CHAIN_CONFIG_ADDRESS)
            .typed(ChainConfigContractProxy)
            .register(new_validator)
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run();

        self.common_setup
            .assert_expected_error_message(result, expect_error);

        self.common_setup
            .assert_expected_log(logs, expected_custom_log);
    }

    pub fn unregister(
        &mut self,
        validator: &ValidatorInfo<StaticApi>,
        expect_error: Option<&str>,
        expected_custom_log: Option<&str>,
    ) {
        let (result, logs) = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(CHAIN_CONFIG_ADDRESS)
            .typed(ChainConfigContractProxy)
            .unregister(validator)
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run();

        self.common_setup
            .assert_expected_error_message(result, expect_error);

        self.common_setup
            .assert_expected_log(logs, expected_custom_log);
    }
}
