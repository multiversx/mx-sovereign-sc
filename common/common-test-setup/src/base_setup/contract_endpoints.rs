use crate::constants::EXECUTED_BRIDGE_OP_EVENT;
use crate::{
    base_setup::init::BaseSetup,
    constants::{CHAIN_CONFIG_ADDRESS, FEE_MARKET_ADDRESS, HEADER_VERIFIER_ADDRESS, OWNER_ADDRESS},
};
use multiversx_sc_scenario::imports::{BigUint, ReturnsResult};
use multiversx_sc_scenario::multiversx_chain_vm::crypto_functions::sha256;
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
use structs::generate_hash::GenerateHash;
use structs::ValidatorData;

impl BaseSetup {
    pub fn register_operation(
        &mut self,
        caller: TestAddress,
        signature: ManagedBuffer<StaticApi>,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        bitmap: ManagedBuffer<StaticApi>,
        epoch: u64,
        operations_hashes: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>>,
    ) {
        self.world
            .tx()
            .from(caller)
            .to(HEADER_VERIFIER_ADDRESS)
            .typed(HeaderverifierProxy)
            .register_bridge_operations(signature, hash_of_hashes, bitmap, epoch, operations_hashes)
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

    pub fn register(
        &mut self,
        bls_key: &ManagedBuffer<StaticApi>,
        payment: &MultiEgldOrEsdtPayment<StaticApi>,
        expected_error_message: Option<&str>,
    ) {
        let (response, _) = self
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
    }

    pub fn unregister(&mut self, bls_key: &ManagedBuffer<StaticApi>, expect_error: Option<&str>) {
        let (response, _) = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(CHAIN_CONFIG_ADDRESS)
            .typed(ChainConfigContractProxy)
            .unregister(bls_key)
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run();

        self.assert_expected_error_message(response, expect_error);
    }

    pub fn register_validator(
        &mut self,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        validator_data: &ValidatorData<StaticApi>,
        expected_custom_log: Option<&str>,
        expected_error_log: Option<&str>,
    ) {
        let (response, logs) = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(CHAIN_CONFIG_ADDRESS)
            .typed(ChainConfigContractProxy)
            .register_validator(hash_of_hashes, validator_data)
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run();

        assert!(response.is_ok());
        self.assert_expected_log(logs, expected_custom_log, expected_error_log);
    }

    pub fn register_validator_operation(
        &mut self,
        validator_data: ValidatorData<StaticApi>,
        signature: ManagedBuffer<StaticApi>,
        bitmap: ManagedBuffer<StaticApi>,
        epoch: u64,
    ) {
        let validator_data_hash = validator_data.generate_hash();
        let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&validator_data_hash.to_vec()));

        self.register_operation(
            OWNER_ADDRESS,
            signature,
            &hash_of_hashes,
            bitmap,
            epoch,
            MultiValueEncoded::from_iter(vec![validator_data_hash]),
        );

        self.register_validator(
            &hash_of_hashes,
            &validator_data,
            Some(EXECUTED_BRIDGE_OP_EVENT),
            None,
        );

        assert_eq!(
            self.get_bls_key_id(&validator_data.bls_key),
            validator_data.id
        );
    }

    pub fn unregister_validator(
        &mut self,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        validator_data: &ValidatorData<StaticApi>,
        expected_error_message: Option<&str>,
        expected_custom_log: Option<&str>,
    ) {
        let (response, logs) = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(CHAIN_CONFIG_ADDRESS)
            .typed(ChainConfigContractProxy)
            .unregister_validator(hash_of_hashes, validator_data)
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run();

        self.assert_expected_error_message(response, expected_error_message);
        self.assert_expected_log(logs, expected_custom_log, None);
    }

    pub fn unregister_validator_operation(
        &mut self,
        validator_data: ValidatorData<StaticApi>,
        signature: ManagedBuffer<StaticApi>,
        bitmap: ManagedBuffer<StaticApi>,
        epoch: u64,
    ) {
        let validator_data_hash = validator_data.generate_hash();
        let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&validator_data_hash.to_vec()));

        self.register_operation(
            OWNER_ADDRESS,
            signature,
            &hash_of_hashes,
            bitmap,
            epoch,
            MultiValueEncoded::from_iter(vec![validator_data_hash]),
        );

        self.unregister_validator(
            &hash_of_hashes,
            &validator_data,
            None,
            Some(EXECUTED_BRIDGE_OP_EVENT),
        );

        assert_eq!(self.get_bls_key_id(&validator_data.bls_key), 0);
    }

    pub fn get_bls_key_id(&mut self, bls_key: &ManagedBuffer<StaticApi>) -> BigUint<StaticApi> {
        self.world
            .query()
            .to(CHAIN_CONFIG_ADDRESS)
            .typed(ChainConfigContractProxy)
            .bls_key_to_id_mapper(bls_key)
            .returns(ReturnsResult)
            .run()
    }
}
