use common_test_setup::base_setup::init::{AccountSetup, BaseSetup};
use common_test_setup::constants::{
    ESDT_SAFE_ADDRESS, HEADER_VERIFIER_ADDRESS, MVX_ESDT_SAFE_CODE_PATH, OWNER_ADDRESS,
    OWNER_BALANCE,
};
use multiversx_sc::api::ManagedTypeApi;
use multiversx_sc::types::{
    BigUint, ManagedBuffer, MultiValueEncoded, ReturnsHandledOrError, TestSCAddress,
};
use multiversx_sc_scenario::imports::*;
use multiversx_sc_scenario::multiversx_chain_vm::crypto_functions::sha256;
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
            address: OWNER_ADDRESS.to_address(),
            code_path: None,
            esdt_balances: None,
            egld_balance: Some(OWNER_BALANCE.into()),
        };

        let mvx_setup = AccountSetup {
            address: ESDT_SAFE_ADDRESS.to_address(),
            code_path: Some(MVX_ESDT_SAFE_CODE_PATH),
            esdt_balances: None,
            egld_balance: Some(OWNER_BALANCE.into()),
        };

        let account_setups = vec![owner_setup, mvx_setup];

        let common_setup = BaseSetup::new(account_setups);

        Self { common_setup }
    }

    pub fn register_operations(
        &mut self,
        operation: BridgeOperation<StaticApi>,
        pub_keys_bitmap: ManagedBuffer<StaticApi>,
        epoch: u64,
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
                pub_keys_bitmap,
                epoch,
                operation.operations_hashes,
            )
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(response, expected_error_message);
    }

    pub fn remove_executed_hash(
        &mut self,
        caller: TestSCAddress,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        operation_hash: &ManagedBuffer<StaticApi>,
        expected_error_message: Option<&str>,
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

        self.common_setup
            .assert_expected_error_message(response, expected_error_message);
    }

    pub fn lock_operation_hash(
        &mut self,
        caller: TestSCAddress,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        operation_hash: &ManagedBuffer<StaticApi>,
        expected_error_message: Option<&str>,
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

        self.common_setup
            .assert_expected_error_message(response, expected_error_message);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn change_validator_set(
        &mut self,
        signature: &ManagedBuffer<StaticApi>,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        operation_hash: &ManagedBuffer<StaticApi>,
        epoch: u64,
        pub_keys_bitmap: &ManagedBuffer<StaticApi>,
        validator_set: MultiValueEncoded<StaticApi, BigUint<StaticApi>>,
        expected_error_message: Option<&str>,
        expected_log: Option<&str>,
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
                pub_keys_bitmap,
                epoch,
                validator_set,
            )
            .returns(ReturnsLogs)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(response, expected_error_message);

        self.common_setup
            .assert_expected_log(logs, expected_log, None);
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

        let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&appended_hashes.to_vec()));

        BridgeOperation {
            signature: ManagedBuffer::new(),
            bridge_operation_hash: hash_of_hashes,
            operations_hashes: bridge_operations,
        }
    }

    // TODO:
    // Cleanup, use the example from chain-config tests
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
