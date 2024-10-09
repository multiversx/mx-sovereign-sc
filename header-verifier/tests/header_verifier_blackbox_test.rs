use bls_signature::BlsSignature;
use header_verifier::{header_verifier_proxy, Headerverifier};
use multiversx_sc::types::ManagedBuffer;
use multiversx_sc::{
    api::ManagedTypeApi,
    types::{BigUint, ManagedByteArray, MultiValueEncoded, TestAddress, TestSCAddress},
};
use multiversx_sc_scenario::{
    api::StaticApi, imports::MxscPath, multiversx_chain_vm::crypto_functions::sha256, DebugApi,
    ExpectError, ScenarioTxRun, ScenarioTxWhitebox, ScenarioWorld,
};

const HEADER_VERIFIER_CODE_PATH: MxscPath = MxscPath::new("ouput/header-verifier.mxsc-json");
const HEADER_VERIFIER_ADDRESS: TestSCAddress = TestSCAddress::new("header-verifier");

// NOTE: This is a mock path
const ENSHRINE_ADDRESS: TestAddress = TestAddress::new("enshrine");

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

        world
            .account(ENSHRINE_ADDRESS)
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

    fn propose_register_esdt_address(&mut self, esdt_address: TestAddress) {
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

    fn propose_remove_executed_hash(
        &mut self,
        caller: TestAddress,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        operation_hash: ManagedBuffer<StaticApi>,
        error_status: Option<ErrorStatus>,
    ) {
        let transaction = self
            .world
            .tx()
            .from(caller)
            .to(HEADER_VERIFIER_ADDRESS)
            .typed(header_verifier_proxy::HeaderverifierProxy)
            .remove_executed_hash(hash_of_hashes, operation_hash);

        match error_status {
            Some(error) => transaction
                .returns(ExpectError(error.code, error.error_message))
                .run(),
            None => transaction.run(),
        }
    }

    fn get_bls_keys(&mut self, bls_keys_vec: Vec<ManagedBuffer<StaticApi>>) -> BlsKeys {
        let bls_keys = bls_keys_vec.iter().map(|key| key).cloned().collect();

        bls_keys
    }

    fn generate_bridge_operation_struct(
        &mut self,
        operation_hashes: Vec<&ManagedBuffer<StaticApi>>,
    ) -> BridgeOperation<StaticApi> {
        let mock_signature: BlsSignature<StaticApi> = ManagedByteArray::new_from_bytes(&[0; 48]);

        let mut bridge_operations: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>> =
            MultiValueEncoded::new();
        let mut appended_hashes = ManagedBuffer::new();

        for operation_hash in operation_hashes {
            appended_hashes.append(&operation_hash);
            bridge_operations.push(operation_hash.clone());
        }

        let hash_of_hashes = self.get_operation_hash(&appended_hashes);

        BridgeOperation {
            signature: mock_signature,
            bridge_operation_hash: hash_of_hashes,
            operations_hashes: bridge_operations,
        }
    }

    fn get_operation_hash(
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
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);

    state.propose_register_operations(operation.clone());

    state
        .world
        .query()
        .to(HEADER_VERIFIER_ADDRESS)
        .whitebox(header_verifier::contract_obj, |sc| {
            let hash_of_hashes: ManagedBuffer<DebugApi> =
                ManagedBuffer::from(operation.bridge_operation_hash.to_vec());

            assert!(!sc.hash_of_hashes_history().is_empty());
            assert!(sc.hash_of_hashes_history().len() == 1);
            assert!(sc.hash_of_hashes_history().contains(&hash_of_hashes));
            assert!(!sc.pending_hashes(&hash_of_hashes).is_empty());

            for (mut i, operation_hash) in operation.operations_hashes.into_iter().enumerate() {
                i += 1;
                let pending_hash = sc.pending_hashes(&hash_of_hashes).get_by_index(i);
                let expected_hash_debug_api: ManagedBuffer<DebugApi> =
                    ManagedBuffer::from(operation_hash.to_vec());
                assert_eq!(pending_hash, expected_hash_debug_api);
            }
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
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);
    let error_status = ErrorStatus {
        code: 4,
        error_message: "Only ESDT Safe contract can call this endpoint",
    };

    state.propose_register_operations(operation.clone());
    state.propose_register_esdt_address(ENSHRINE_ADDRESS);
    state.propose_remove_executed_hash(
        OWNER,
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
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);

    let error_status = ErrorStatus {
        code: 4,
        error_message: "There is no registered ESDT address",
    };

    state.propose_register_operations(operation.clone());
    state.propose_remove_executed_hash(
        ENSHRINE_ADDRESS,
        &operation.bridge_operation_hash,
        operation_1,
        Some(error_status),
    );
}

#[test]
fn test_remove_one_executed_hash() {
    let mut state = HeaderVerifierTestState::new();
    let bls_key_1 = ManagedBuffer::from("bls_key_1");
    let managed_bls_keys = state.get_bls_keys(vec![bls_key_1]);

    state.deploy_header_verifier_contract(managed_bls_keys);

    let operation_hash_1 = ManagedBuffer::from("operation_1");
    let operation_hash_2 = ManagedBuffer::from("operation_2");
    let operation =
        state.generate_bridge_operation_struct(vec![&operation_hash_1, &operation_hash_2]);

    state.propose_register_operations(operation.clone());
    state.propose_register_esdt_address(ENSHRINE_ADDRESS);

    state.propose_remove_executed_hash(
        ENSHRINE_ADDRESS,
        &operation.bridge_operation_hash,
        operation_hash_1,
        None,
    );

    state
        .world
        .query()
        .to(HEADER_VERIFIER_ADDRESS)
        .whitebox(header_verifier::contract_obj, |sc| {
            let hash_of_hashes: ManagedBuffer<DebugApi> =
                ManagedBuffer::from(operation.bridge_operation_hash.to_vec());
            assert!(!sc.pending_hashes(&hash_of_hashes).is_empty());
            assert!(sc.pending_hashes(&hash_of_hashes).len() == 1);

            let pending_hash_2 = sc.pending_hashes(&hash_of_hashes).get_by_index(1);
            let expected_hash_2_debug_api: ManagedBuffer<DebugApi> =
                ManagedBuffer::from(operation_hash_2.to_vec());

            assert_eq!(pending_hash_2, expected_hash_2_debug_api);
        });
}

#[test]
fn test_remove_all_executed_hashes() {
    let mut state = HeaderVerifierTestState::new();
    let bls_key_1 = ManagedBuffer::from("bls_key_1");
    let managed_bls_keys = state.get_bls_keys(vec![bls_key_1]);

    state.deploy_header_verifier_contract(managed_bls_keys);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);

    state.propose_register_operations(operation.clone());
    state.propose_register_esdt_address(ENSHRINE_ADDRESS);

    state.propose_remove_executed_hash(
        ENSHRINE_ADDRESS,
        &operation.bridge_operation_hash,
        operation_1,
        None,
    );

    state.propose_remove_executed_hash(
        ENSHRINE_ADDRESS,
        &operation.bridge_operation_hash,
        operation_2,
        None,
    );
    state
        .world
        .query()
        .to(HEADER_VERIFIER_ADDRESS)
        .whitebox(header_verifier::contract_obj, |sc| {
            let hash_of_hashes: ManagedBuffer<DebugApi> =
                ManagedBuffer::from(operation.bridge_operation_hash.to_vec());
            assert!(sc.pending_hashes(&hash_of_hashes).is_empty());
            assert!(sc.hash_of_hashes_history().contains(&hash_of_hashes));
        });
}
