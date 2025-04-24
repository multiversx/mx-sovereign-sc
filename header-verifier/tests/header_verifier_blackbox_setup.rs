use common_test_setup::constants::{
    ENSHRINE_ADDRESS, HEADER_VERIFIER_ADDRESS, OWNER_ADDRESS, OWNER_BALANCE,
};
use common_test_setup::{AccountSetup, BaseSetup};
use multiversx_sc::api::ManagedTypeApi;
use multiversx_sc::types::{ManagedBuffer, MultiValueEncoded, TestAddress};
use multiversx_sc_scenario::{
    api::StaticApi, multiversx_chain_vm::crypto_functions::sha256, ScenarioTxRun,
};
use multiversx_sc_scenario::{ReturnsHandledOrError, ReturnsLogs};
use proxies::header_verifier_proxy::HeaderverifierProxy;

#[derive(Clone)]
pub struct BridgeOperation<M: ManagedTypeApi> {
    pub signature: ManagedBuffer<M>,
    pub bridge_operation_hash: ManagedBuffer<M>,
    pub operations_hashes: MultiValueEncoded<M, ManagedBuffer<M>>,
}

pub struct HeaderVerifierTestState {
    pub common_setup: BaseSetup,
}

impl HeaderVerifierTestState {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let owner_setup = AccountSetup {
            address: OWNER_ADDRESS,
            esdt_balances: None,
            egld_balance: Some(OWNER_BALANCE.into()),
        };

        let enshrine_setup = AccountSetup {
            address: ENSHRINE_ADDRESS,
            esdt_balances: None,
            egld_balance: Some(OWNER_BALANCE.into()),
        };

        let account_setups = vec![owner_setup, enshrine_setup];

        let common_setup = BaseSetup::new(account_setups);

        Self { common_setup }
    }

    pub fn register_operations(
        &mut self,
        operation: BridgeOperation<StaticApi>,
        expected_error_message: Option<&str>,
    ) {
        let response = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(HEADER_VERIFIER_ADDRESS)
            .typed(HeaderverifierProxy)
            .register_bridge_operations(
                operation.signature,
                operation.bridge_operation_hash,
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                operation.operations_hashes,
            )
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(response, expected_error_message);
    }

    pub fn complete_setup_phase(&mut self, expected_error_message: Option<&str>) {
        let response = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(HEADER_VERIFIER_ADDRESS)
            .typed(HeaderverifierProxy)
            .complete_setup_phase()
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(response, expected_error_message);
    }

    pub fn register_esdt_address(&mut self, esdt_address: TestAddress) {
        self.common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(HEADER_VERIFIER_ADDRESS)
            .typed(HeaderverifierProxy)
            .set_esdt_safe_address(esdt_address)
            .run();
    }

    pub fn remove_executed_hash(
        &mut self,
        caller: TestAddress,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        operation_hash: &ManagedBuffer<StaticApi>,
        expected_result: Option<&str>,
    ) {
        let response = self
            .common_setup
            .world
            .tx()
            .from(caller)
            .to(HEADER_VERIFIER_ADDRESS)
            .typed(HeaderverifierProxy)
            .remove_executed_hash(hash_of_hashes, operation_hash)
            .returns(ReturnsHandledOrError::new())
            .run();

        match response {
            Ok(_) => assert!(
                expected_result.is_none(),
                "Transaction was successful, but expected error"
            ),
            Err(error) => assert_eq!(expected_result, Some(error.message.as_str())),
        };
    }

    pub fn lock_operation_hash(
        &mut self,
        caller: TestAddress,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        operation_hash: &ManagedBuffer<StaticApi>,
        expected_result: Option<&str>,
    ) {
        let response = self
            .common_setup
            .world
            .tx()
            .from(caller)
            .to(HEADER_VERIFIER_ADDRESS)
            .typed(HeaderverifierProxy)
            .lock_operation_hash(hash_of_hashes, operation_hash)
            .returns(ReturnsHandledOrError::new())
            .run();

        match response {
            Ok(_) => assert!(
                expected_result.is_none(),
                "Transaction was successful, but expected error"
            ),
            Err(error) => assert_eq!(expected_result, Some(error.message.as_str())),
        };
    }

    pub fn change_validator_set(
        &mut self,
        signature: &ManagedBuffer<StaticApi>,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        operation_hash: &ManagedBuffer<StaticApi>,
        expected_error_message: Option<&str>,
        expected_custom_log: Option<&str>,
    ) {
        let (logs, response) = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(HEADER_VERIFIER_ADDRESS)
            .typed(HeaderverifierProxy)
            .change_validator_set(
                signature,
                hash_of_hashes,
                operation_hash,
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                MultiValueEncoded::new(),
            )
            .returns(ReturnsLogs)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(response, expected_error_message);

        if let Some(custom_log) = expected_custom_log {
            self.common_setup.assert_expected_log(logs, custom_log)
        };
    }

    pub fn generate_bridge_operation_struct(
        &mut self,
        operation_hashes: Vec<&ManagedBuffer<StaticApi>>,
    ) -> BridgeOperation<StaticApi> {
        let mut bridge_operations: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>> =
            MultiValueEncoded::new();
        let mut appended_hashes = ManagedBuffer::new();

        for operation_hash in operation_hashes {
            appended_hashes.append(operation_hash);
            bridge_operations.push(operation_hash.clone());
        }

        let hash_of_hashes = self.get_operation_hash(&appended_hashes);

        BridgeOperation {
            signature: ManagedBuffer::new(),
            bridge_operation_hash: hash_of_hashes,
            operations_hashes: bridge_operations,
        }
    }

    pub fn get_operation_hash(
        &mut self,
        operation: &ManagedBuffer<StaticApi>,
    ) -> ManagedBuffer<StaticApi> {
        let mut array = [0; 1024];

        let len = {
            let byte_array = operation.load_to_byte_array(&mut array);
            byte_array.len()
        };

        let trimmed_slice = &array[..len];
        let hash = sha256(trimmed_slice);

        ManagedBuffer::from(&hash)
    }
}
