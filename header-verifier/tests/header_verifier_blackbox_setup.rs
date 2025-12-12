use common_test_setup::base_setup::init::{AccountSetup, BaseSetup, ExpectedLogs};
use common_test_setup::base_setup::log_validations::assert_expected_logs;
use common_test_setup::constants::{
    CHANGE_VALIDATOR_SET_ENDPOINT, ESDT_SAFE_ADDRESS, EXECUTED_BRIDGE_OP_EVENT,
    HEADER_VERIFIER_ADDRESS, MVX_ESDT_SAFE_CODE_PATH, OWNER_ADDRESS, OWNER_BALANCE,
};
use common_test_setup::log;
use header_verifier::storage::HeaderVerifierStorageModule;
use multiversx_sc::api::ManagedTypeApi;
use multiversx_sc::types::{
    BigUint, ManagedBuffer, MultiValueEncoded, ReturnsHandledOrError, ReturnsResult,
    ReturnsResultUnmanaged, TestSCAddress,
};
use multiversx_sc_scenario::imports::*;
use multiversx_sc_scenario::multiversx_chain_vm::crypto_functions::sha256;
use multiversx_sc_scenario::DebugApi;
use proxies::header_verifier_proxy::HeaderverifierProxy;
use structs::aliases::TxNonce;

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
        signature: &ManagedBuffer<StaticApi>,
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
                signature,
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

    pub fn with_header_verifier<F>(&mut self, f: F)
    where
        F: FnOnce(header_verifier::ContractObj<DebugApi>),
    {
        self.common_setup
            .world
            .query()
            .to(HEADER_VERIFIER_ADDRESS)
            .whitebox(header_verifier::contract_obj, f);
    }

    pub fn last_operation_nonce(&mut self) -> TxNonce {
        let mut nonce: TxNonce = 0;
        self.with_header_verifier(|sc| {
            let current = sc.current_execution_nonce().get();
            nonce = current.saturating_sub(1);
        });
        nonce
    }

    pub fn next_operation_nonce(&mut self) -> TxNonce {
        self.common_setup.next_operation_nonce()
    }

    pub fn assert_last_operation_nonce(&mut self, expected: TxNonce) {
        let actual = self.last_operation_nonce();
        assert_eq!(actual, expected);
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
            .returns(ReturnsResult)
            .run()
            .into_option();

        self.common_setup
            .assert_optional_error_message(response, expected_error_message);
    }

    pub fn lock_operation_hash(
        &mut self,
        caller: TestSCAddress,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        operation_hash: &ManagedBuffer<StaticApi>,
        operation_nonce: TxNonce,
        expected_error_message: Option<&str>,
    ) {
        let response = self
            .common_setup
            .world
            .tx()
            .from(caller)
            .to(HEADER_VERIFIER_ADDRESS)
            .typed(HeaderverifierProxy)
            .lock_operation_hash(hash_of_hashes, operation_hash, operation_nonce)
            .returns(ReturnsResultUnmanaged)
            .run()
            .into_option();

        self.common_setup
            .assert_optional_error_message(response, expected_error_message);
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
        execution_error: Option<&str>,
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
            .assert_expected_error_message(response, None);

        let expected_logs = vec![
            log!(CHANGE_VALIDATOR_SET_ENDPOINT, topics: [EXECUTED_BRIDGE_OP_EVENT], data: execution_error),
        ];

        assert_expected_logs(logs, expected_logs);
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
}
