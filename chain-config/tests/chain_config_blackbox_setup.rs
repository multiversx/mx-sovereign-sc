use common_test_setup::{
    base_setup::init::{AccountSetup, BaseSetup},
    constants::{
        CHAIN_CONFIG_ADDRESS, FIRST_TEST_TOKEN, ONE_HUNDRED_MILLION, OWNER_ADDRESS, OWNER_BALANCE,
        USER_ADDRESS,
    },
};
use multiversx_sc::types::{
    BigUint, ManagedBuffer, MultiEgldOrEsdtPayment, MultiValueEncoded, ReturnsResult, TestAddress,
};
use multiversx_sc_scenario::{
    api::StaticApi, multiversx_chain_vm::crypto_functions::sha256, ReturnsHandledOrError,
    ReturnsLogs, ScenarioTxRun,
};
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
        expected_log_error: Option<&str>,
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
            .assert_expected_log(logs, expected_custom_log, expected_log_error);
    }

    pub fn register(
        &mut self,
        bls_key: &ManagedBuffer<StaticApi>,
        payment: &MultiEgldOrEsdtPayment<StaticApi>,
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
            .register(bls_key)
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .payment(payment)
            .run();

        self.common_setup
            .assert_expected_error_message(result, expect_error);

        self.common_setup
            .assert_expected_log(logs, expected_custom_log);
    }

    pub fn unregister_with_caller(
        &mut self,
        bls_key: &ManagedBuffer<StaticApi>,
        caller: TestAddress,
        expect_error: Option<&str>,
        expected_custom_log: Option<&str>,
    ) {
        let (result, logs) = self
            .common_setup
            .world
            .tx()
            .from(caller)
            .to(CHAIN_CONFIG_ADDRESS)
            .typed(ChainConfigContractProxy)
            .unregister(bls_key)
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
        bls_key: &ManagedBuffer<StaticApi>,
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
            .unregister(bls_key)
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run();

        self.common_setup
            .assert_expected_error_message(result, expect_error);

        self.common_setup
            .assert_expected_log(logs, expected_custom_log);
    }

    pub fn get_bls_key_id(&mut self, bls_key: &ManagedBuffer<StaticApi>) -> BigUint<StaticApi> {
        self.common_setup
            .world
            .query()
            .to(CHAIN_CONFIG_ADDRESS)
            .typed(ChainConfigContractProxy)
            .bls_key_to_id_mapper(bls_key)
            .returns(ReturnsResult)
            .run()
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

    pub fn register_and_update_registration_status(
        &mut self,
        registration_status: u8,
        signature: ManagedBuffer<StaticApi>,
        bitmap: ManagedBuffer<StaticApi>,
        epoch: u64,
    ) {
        let new_status_hash_byte_array = sha256(&[registration_status]);
        let new_status_hash = ManagedBuffer::new_from_bytes(&new_status_hash_byte_array);
        let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&new_status_hash_byte_array));

        self.common_setup.register_operation(
            OWNER_ADDRESS,
            signature,
            &hash_of_hashes,
            bitmap,
            epoch,
            MultiValueEncoded::from_iter(vec![new_status_hash]),
        );

        self.common_setup.update_registration_status(
            &hash_of_hashes,
            1,
            None,
            Some("registrationStatusUpdate"),
        );
    }
}
