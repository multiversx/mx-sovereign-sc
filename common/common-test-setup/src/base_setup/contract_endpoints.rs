use multiversx_sc_scenario::{
    api::StaticApi,
    imports::{
        ManagedBuffer, MultiEgldOrEsdtPayment, MultiValueEncoded, ReturnsHandledOrError,
        TestAddress,
    },
    ReturnsLogs, ScenarioTxRun,
};
use proxies::{
    chain_config_proxy::ChainConfigContractProxy, fee_market_proxy::FeeMarketProxy,
    header_verifier_proxy::HeaderverifierProxy,
};
use structs::fee::FeeStruct;

use crate::{
    base_setup::init::BaseSetup,
    constants::{CHAIN_CONFIG_ADDRESS, FEE_MARKET_ADDRESS, HEADER_VERIFIER_ADDRESS, OWNER_ADDRESS},
};

impl BaseSetup {
    pub fn register_operation(
        &mut self,
        caller: TestAddress,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        operations_hashes: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>>,
    ) {
        let signature = ManagedBuffer::new();
        let bls_keys_bitmap = ManagedBuffer::new_from_bytes(&[1]);
        let epoch = 0u64;
        self.world
            .tx()
            .from(caller)
            .to(HEADER_VERIFIER_ADDRESS)
            .typed(HeaderverifierProxy)
            .register_bridge_operations(
                signature,
                hash_of_hashes,
                bls_keys_bitmap,
                epoch,
                operations_hashes,
            )
            .run();
    }

    pub fn set_fee_during_setup_phase(
        &mut self,
        fee_struct: FeeStruct<StaticApi>,
        error_message: Option<&str>,
    ) {
        let response = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(FEE_MARKET_ADDRESS)
            .typed(FeeMarketProxy)
            .set_fee_during_setup_phase(fee_struct)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.assert_expected_error_message(response, error_message);
    }

    pub fn set_fee(
        &mut self,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        fee_struct: Option<FeeStruct<StaticApi>>,
        error_message: Option<&str>,
    ) {
        let response = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(FEE_MARKET_ADDRESS)
            .typed(FeeMarketProxy)
            .set_fee(hash_of_hashes, fee_struct.unwrap())
            .returns(ReturnsHandledOrError::new())
            .run();

        self.assert_expected_error_message(response, error_message);
    }

    pub fn update_registration_status(
        &mut self,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        registration_status: u8,
        expected_error_message: Option<&str>,
        expected_log: Option<&str>,
        expected_log_error: Option<&str>,
    ) {
        let (response, logs) = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(CHAIN_CONFIG_ADDRESS)
            .typed(ChainConfigContractProxy)
            .update_registration_status(hash_of_hashes, registration_status)
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run();

        self.assert_expected_error_message(response, expected_error_message);
        self.assert_expected_log(logs, expected_log, expected_log_error);
    }

    // TODO: Use this for any validator registration
    pub fn register_as_validator(
        &mut self,
        bls_key: &ManagedBuffer<StaticApi>,
        payment: &MultiEgldOrEsdtPayment<StaticApi>,
        expected_error_message: Option<&str>,
        expected_custom_log: Option<&str>,
    ) {
        let (response, logs) = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(CHAIN_CONFIG_ADDRESS)
            .typed(ChainConfigContractProxy)
            .register(bls_key)
            .payment(payment)
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run();

        self.assert_expected_error_message(response, expected_error_message);
        self.assert_expected_log(logs, expected_custom_log, None);
    }

    pub fn register(
        &mut self,
        bls_key: &ManagedBuffer<StaticApi>,
        payment: &MultiEgldOrEsdtPayment<StaticApi>,
        expect_error: Option<&str>,
        expected_custom_log: Option<&str>,
    ) {
        let (result, logs) = self
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

        self.assert_expected_error_message(result, expect_error);

        self.assert_expected_log(logs, expected_custom_log, None);
    }
}
