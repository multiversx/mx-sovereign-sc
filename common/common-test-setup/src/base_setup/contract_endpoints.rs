use multiversx_sc_scenario::{
    api::StaticApi,
    imports::{ManagedBuffer, MultiValueEncoded, TestAddress},
    scenario_model::Log,
    ReturnsHandledOrError, ReturnsLogs, ScenarioTxRun,
};
use proxies::{
    chain_config_proxy::ChainConfigContractProxy, fee_market_proxy::FeeMarketProxy,
    header_verifier_proxy::HeaderverifierProxy,
};
use structs::{fee::FeeStruct, ValidatorInfo};

use crate::{
    base_setup::init::BaseSetup,
    constants::{CHAIN_CONFIG_ADDRESS, FEE_MARKET_ADDRESS, HEADER_VERIFIER_ADDRESS, OWNER_ADDRESS},
};

impl BaseSetup {
    pub fn register_operation(
        &mut self,
        caller: TestAddress,
        signature: ManagedBuffer<StaticApi>,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        bls_keys_bitmap: ManagedBuffer<StaticApi>,
        epoch: u64,
        operations_hashes: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>>,
    ) {
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

    pub fn register_validator(
        &mut self,
        new_validator: ValidatorInfo<StaticApi>,
        expected_error_message: Option<&str>,
        expected_log: Option<&str>,
    ) {
        let (response, logs) = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(CHAIN_CONFIG_ADDRESS)
            .typed(ChainConfigContractProxy)
            .register(new_validator.bls_key)
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run();

        self.assert_expected_error_message(response, expected_error_message);
        self.assert_expected_log(logs, expected_log);
    }
}
