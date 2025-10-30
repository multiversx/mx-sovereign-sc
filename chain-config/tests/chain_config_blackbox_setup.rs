use common_test_setup::base_setup::init::ExpectedLogs;
use common_test_setup::{
    base_setup::init::{AccountSetup, BaseSetup},
    constants::{
        CHAIN_CONFIG_ADDRESS, EXECUTED_BRIDGE_OP_EVENT, FIRST_TEST_TOKEN, ONE_HUNDRED_MILLION,
        OWNER_ADDRESS, OWNER_BALANCE, UPDATE_SOVEREIGN_CONFIG_ENDPOINT, USER_ADDRESS,
    },
    log,
};
use multiversx_sc::types::{
    BigUint, ManagedBuffer, ReturnsHandledOrError, ReturnsResult, TestAddress,
};
use multiversx_sc_scenario::imports::OptionalValue;
use multiversx_sc_scenario::{api::StaticApi, ReturnsLogs, ScenarioTxRun};
use proxies::chain_config_proxy::ChainConfigContractProxy;
use structs::{
    aliases::TxNonce,
    configs::{SovereignConfig, UpdateSovereignConfigOperation},
};

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
            esdt_balances: Some(vec![(FIRST_TEST_TOKEN, 0u64, ONE_HUNDRED_MILLION.into())]),
        };

        let user_account = AccountSetup {
            address: USER_ADDRESS.to_address(),
            code_path: None,
            egld_balance: Some(OWNER_BALANCE.into()),
            esdt_balances: Some(vec![(FIRST_TEST_TOKEN, 0u64, ONE_HUNDRED_MILLION.into())]),
        };

        let account_setups = vec![owner_account, user_account];

        let common_setup = BaseSetup::new(account_setups);

        Self { common_setup }
    }

    pub fn update_sovereign_config_during_setup_phase(
        &mut self,
        config: SovereignConfig<StaticApi>,
        expected_error_message: Option<&str>,
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
            .assert_expected_error_message(result, expected_error_message);
    }

    pub fn update_sovereign_config(
        &mut self,
        hash_of_hashes: ManagedBuffer<StaticApi>,
        config: SovereignConfig<StaticApi>,
        operation_nonce: TxNonce,
        expected_error_message: Option<&str>,
    ) {
        let (result, logs) = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(CHAIN_CONFIG_ADDRESS)
            .typed(ChainConfigContractProxy)
            .update_sovereign_config(
                hash_of_hashes,
                UpdateSovereignConfigOperation {
                    sovereign_config: config,
                    nonce: operation_nonce,
                },
            )
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run();

        self.common_setup
            .assert_expected_error_message(result, None);

        let expected_logs = vec![
            log!(UPDATE_SOVEREIGN_CONFIG_ENDPOINT, topics: [EXECUTED_BRIDGE_OP_EVENT], data: expected_error_message),
        ];

        self.common_setup.assert_expected_logs(logs, expected_logs);
    }

    pub fn unregister_with_caller(
        &mut self,
        bls_key: &ManagedBuffer<StaticApi>,
        caller: TestAddress,
        expect_error: Option<&str>,
    ) {
        let result = self
            .common_setup
            .world
            .tx()
            .from(caller)
            .to(CHAIN_CONFIG_ADDRESS)
            .typed(ChainConfigContractProxy)
            .unregister(bls_key)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(result, expect_error);
    }

    pub fn get_bls_key_by_id(&mut self, id: &BigUint<StaticApi>) -> ManagedBuffer<StaticApi> {
        let (_, bls_key) = self
            .common_setup
            .world
            .query()
            .to(CHAIN_CONFIG_ADDRESS)
            .typed(ChainConfigContractProxy)
            .bls_keys_map()
            .returns(ReturnsResult)
            .run()
            .into_iter()
            .find(|v| {
                let (returned_id, _) = v.clone().into_tuple();

                returned_id.eq(id)
            })
            .unwrap()
            .into_tuple();

        bls_key
    }
}
