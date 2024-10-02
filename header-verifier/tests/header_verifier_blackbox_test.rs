use bls_signature::BlsSignature;
use header_verifier::{header_verifier_proxy, Headerverifier};
use multiversx_sc::{
    api::ManagedTypeApi,
    types::{
        BigUint, ManagedBuffer, ManagedByteArray, MultiValueEncoded, TestAddress, TestSCAddress,
    },
};
use multiversx_sc_scenario::{
    api::StaticApi, imports::MxscPath, multiversx_chain_vm::crypto_functions::sha256, DebugApi,
    ExpectError, ScenarioTxRun, ScenarioTxWhitebox, ScenarioWorld,
};

const HEADER_VERIFIER_CODE_PATH: MxscPath = MxscPath::new("ouput/header-verifier.mxsc-json");
const HEADER_VERIFIER_ADDRESS: TestSCAddress = TestSCAddress::new("header-verifier");

// NOTE: This is a mock path
const ENSHRINE_CODE_PATH: MxscPath =
    MxscPath::new("../chain-factory/output/chain-factory.mxsc-json");
const ENSHRINE_ADDRESS: TestSCAddress = TestSCAddress::new("enshrine");

const OWNER: TestAddress = TestAddress::new("owner");
const WEGLD_BALANCE: u128 = 100_000_000_000_000_000;

type BlsKeys = MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>>;

pub struct ErrorStatus<'a> {
    code: u64,
    error_message: &'a str,
}

#[derive(Clone)]
pub struct BridgeOperation<M: ManagedTypeApi> {
    signature: BlsSignature<M>,
    bridge_operation_hash: ManagedBuffer<M>,
    operations_hashes: MultiValueEncoded<M, ManagedBuffer<M>>,
}

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();
    blockchain.register_contract(HEADER_VERIFIER_CODE_PATH, header_verifier::ContractBuilder);
    blockchain.register_contract(ENSHRINE_CODE_PATH, header_verifier::ContractBuilder);

    blockchain
}

struct HeaderVerifierTestState {
    world: ScenarioWorld,
}

impl HeaderVerifierTestState {
    fn new() -> Self {
        let mut world = world();

        world
            .account(OWNER)
            .balance(BigUint::from(WEGLD_BALANCE))
            .nonce(1);

        Self { world }
    }

    fn deploy_header_verifier_contract(&mut self, bls_keys: BlsKeys) -> &mut Self {
        self.world
            .tx()
            .from(OWNER)
            .typed(header_verifier_proxy::HeaderverifierProxy)
            .init(bls_keys)
            .code(HEADER_VERIFIER_CODE_PATH)
            .new_address(HEADER_VERIFIER_ADDRESS)
            .run();

        self
    }

    fn deploy_enshrine_esdt_contract(&mut self, bls_keys: &BlsKeys) -> &mut Self {
        self.world
            .tx()
            .from(OWNER)
            .typed(header_verifier_proxy::HeaderverifierProxy)
            .init(bls_keys)
            .code(HEADER_VERIFIER_CODE_PATH)
            .new_address(ENSHRINE_ADDRESS)
            .run();

        self
    }

    fn propose_register_esdt_address(&mut self, esdt_address: TestSCAddress) {
        self.world
            .tx()
            .from(OWNER)
            .to(HEADER_VERIFIER_ADDRESS)
            .typed(header_verifier_proxy::HeaderverifierProxy)
            .set_esdt_safe_address(esdt_address)
            .run();
    }

    fn propose_register_operations(&mut self, operation: BridgeOperation<StaticApi>) {
        self.world
            .tx()
            .from(OWNER)
            .to(HEADER_VERIFIER_ADDRESS)
            .typed(header_verifier_proxy::HeaderverifierProxy)
            .register_bridge_operations(
                operation.signature,
                operation.bridge_operation_hash,
                operation.operations_hashes,
            )
            .run();
    }

    fn propose_remove_execute_hash(
        &mut self,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        operation_hash: ManagedBuffer<StaticApi>,
        error_status: Option<ErrorStatus>,
    ) {
        match error_status {
            Some(error) => self
                .world
                .tx()
                .from(OWNER)
                .to(HEADER_VERIFIER_ADDRESS)
                .typed(header_verifier_proxy::HeaderverifierProxy)
                .remove_executed_hash(hash_of_hashes, operation_hash)
                .returns(ExpectError(error.code, error.error_message))
                .run(),
            None => self
                .world
                .tx()
                .from(ENSHRINE_ADDRESS)
                .to(HEADER_VERIFIER_ADDRESS)
                .typed(header_verifier_proxy::HeaderverifierProxy)
                .remove_executed_hash(hash_of_hashes, operation_hash)
                .run(),
        }
    }

    fn get_bls_keys(&mut self, bls_keys_vec: Vec<ManagedBuffer<StaticApi>>) -> BlsKeys {
        let bls_keys = bls_keys_vec.iter().map(|key| key.clone().into()).collect();

        bls_keys
    }

    fn get_mock_operation(
        &mut self,
        operations: Vec<&ManagedBuffer<StaticApi>>,
    ) -> BridgeOperation<StaticApi> {
        let mock_signature: BlsSignature<StaticApi> = ManagedByteArray::new_from_bytes(&[0; 48]);

        let mut bridge_operations: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>> =
            MultiValueEncoded::new();
        let mut bridge_operation_appended_hashes = ManagedBuffer::new();

        for operation in operations {
            let operation_hash = self.get_operation_hash(operation);
            bridge_operation_appended_hashes.append(&operation_hash);
            bridge_operations.push(operation_hash);
        }

        let bridge_operations_hash = self.get_operation_hash(&bridge_operation_appended_hashes);

        BridgeOperation {
            signature: mock_signature,
            bridge_operation_hash: bridge_operations_hash,
            operations_hashes: bridge_operations,
        }
    }

    fn get_operation_hash(
        &mut self,
        operation: &ManagedBuffer<StaticApi>,
    ) -> ManagedBuffer<StaticApi> {
        let array: &mut [u8; 64] = &mut [0u8; 64];
        operation.load_to_byte_array(array);

        ManagedBuffer::from(&sha256(array))
    }
}

#[test]
fn test_deploy() {
    let mut state = HeaderVerifierTestState::new();
    let bls_key_1 = ManagedBuffer::from("bls_key_1");
    let managed_bls_keys = state.get_bls_keys(vec![bls_key_1]);

    state.deploy_header_verifier_contract(managed_bls_keys);
}

#[test]
fn test_register_esdt_address() {
    let mut state = HeaderVerifierTestState::new();
    let bls_key_1 = ManagedBuffer::from("bls_key_1");
    let managed_bls_keys = state.get_bls_keys(vec![bls_key_1]);

    state.deploy_header_verifier_contract(managed_bls_keys);
    state.propose_register_esdt_address(ENSHRINE_ADDRESS);

    state
        .world
        .query()
        .to(HEADER_VERIFIER_ADDRESS)
        .whitebox(header_verifier::contract_obj, |sc| {
            let esdt_address = sc.esdt_safe_address().get();

            assert_eq!(esdt_address, ENSHRINE_ADDRESS);
        })
}

#[test]
fn test_register_bridge_operation() {
    let mut state = HeaderVerifierTestState::new();
    let bls_key_1 = ManagedBuffer::from("bls_key_1");
    let managed_bls_keys = state.get_bls_keys(vec![bls_key_1]);

    state.deploy_header_verifier_contract(managed_bls_keys);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.get_mock_operation(vec![&operation_1, &operation_2]);

    state.propose_register_operations(operation.clone());

    let expected_hash_1 = state.get_operation_hash(&operation_1);
    let expected_hash_2 = state.get_operation_hash(&operation_2);

    state
        .world
        .query()
        .to(HEADER_VERIFIER_ADDRESS)
        .whitebox(header_verifier::contract_obj, |sc| {
            let hash_of_hashes: ManagedBuffer<DebugApi> =
                ManagedBuffer::from(operation.bridge_operation_hash.to_vec());

            assert!(!sc.hash_of_hashes_history().is_empty());
            assert!(!sc.pending_hashes(&hash_of_hashes).is_empty());

            let pending_hash_1 = sc.pending_hashes(&hash_of_hashes).get_by_index(1);
            let pending_hash_2 = sc.pending_hashes(&hash_of_hashes).get_by_index(2);

            let expected_hash_1_debug_api: ManagedBuffer<DebugApi> =
                ManagedBuffer::from(expected_hash_1.to_vec());
            let expected_hash_2_debug_api: ManagedBuffer<DebugApi> =
                ManagedBuffer::from(expected_hash_2.to_vec());

            assert_eq!(pending_hash_1, expected_hash_1_debug_api);
            assert_eq!(pending_hash_2, expected_hash_2_debug_api);
        });
}

#[test]
fn test_remove_executed_hash_caller_not_esdt_address() {
    let mut state = HeaderVerifierTestState::new();
    let bls_key_1 = ManagedBuffer::from("bls_key_1");
    let managed_bls_keys = state.get_bls_keys(vec![bls_key_1]);

    state.deploy_header_verifier_contract(managed_bls_keys);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.get_mock_operation(vec![&operation_1, &operation_2]);

    let error_status = ErrorStatus {
        code: 4,
        error_message: "Only ESDT Safe contract can call this endpoint",
    };

    state.propose_register_operations(operation.clone());
    state.propose_register_esdt_address(ENSHRINE_ADDRESS);
    state.propose_remove_execute_hash(
        &operation.bridge_operation_hash,
        operation_1,
        Some(error_status),
    );
}

#[test]
fn test_remove_executed_hash_no_esdt_address_registered() {
    let mut state = HeaderVerifierTestState::new();
    let bls_key_1 = ManagedBuffer::from("bls_key_1");
    let managed_bls_keys = state.get_bls_keys(vec![bls_key_1]);

    state.deploy_header_verifier_contract(managed_bls_keys);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.get_mock_operation(vec![&operation_1, &operation_2]);

    let error_status = ErrorStatus {
        code: 4,
        error_message: "There is no registered ESDT address",
    };

    state.propose_register_operations(operation.clone());
    state.propose_remove_execute_hash(
        &operation.bridge_operation_hash,
        operation_1,
        Some(error_status),
    );
}

#[test]
fn test_remove_executed_hash() {
    let mut state = HeaderVerifierTestState::new();
    let bls_key_1 = ManagedBuffer::from("bls_key_1");
    let managed_bls_keys = state.get_bls_keys(vec![bls_key_1]);

    state.deploy_enshrine_esdt_contract(&managed_bls_keys);
    state.deploy_header_verifier_contract(managed_bls_keys);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.get_mock_operation(vec![&operation_1, &operation_2]);

    state.propose_register_operations(operation.clone());
    state.propose_register_esdt_address(ENSHRINE_ADDRESS);
    state.propose_remove_execute_hash(&operation.bridge_operation_hash, operation_1, None);

    state
        .world
        .query()
        .to(HEADER_VERIFIER_ADDRESS)
        .whitebox(header_verifier::contract_obj, |sc| {
            let hash_of_hashes: ManagedBuffer<DebugApi> =
                ManagedBuffer::from(operation.bridge_operation_hash.to_vec());
            sc.pending_hashes(&hash_of_hashes).is_empty();
        });
}
