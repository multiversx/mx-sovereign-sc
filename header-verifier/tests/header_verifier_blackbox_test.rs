use bls_signature::BlsSignature;
use header_verifier::{header_verifier_proxy, Headerverifier};
use multiversx_sc::{
    api::ManagedTypeApi,
    types::{
        BigUint, ManagedBuffer, ManagedByteArray, MultiValueEncoded, TestAddress, TestSCAddress,
    },
};
use multiversx_sc_scenario::{
    api::StaticApi, imports::MxscPath, multiversx_chain_vm::crypto_functions::sha256,
    ScenarioTxRun, ScenarioTxWhitebox, ScenarioWorld,
};

const HEADER_VERIFIER_CODE_PATH: MxscPath = MxscPath::new("ouput/header-verifier.mxsc-json");
const HEADER_VERIFIER_ADDRESS: TestSCAddress = TestSCAddress::new("header-verifier");

// NOTE: This is a mock path
const ENSHRINE_CODE_PATH: MxscPath =
    MxscPath::new("../chain-factory/output/chain-factory.mxsc-json");
const ENSHRINE_ADDRESS: TestSCAddress = TestSCAddress::new("enshrine");

const OWNER: TestAddress = TestAddress::new("owner");
const LEADER: TestAddress = TestAddress::new("leader");
const VALIDATOR: TestAddress = TestAddress::new("validator");
const WEGLD_BALANCE: u128 = 100_000_000_000_000_000;

type BlsKeys = MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>>;

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

    fn get_bls_keys(&mut self, bls_keys_vec: Vec<ManagedBuffer<StaticApi>>) -> BlsKeys {
        let bls_keys = bls_keys_vec.iter().map(|key| key.clone().into()).collect();

        bls_keys
    }

    fn get_mock_operation(&mut self) -> BridgeOperation<StaticApi> {
        let mock_signature: BlsSignature<StaticApi> = ManagedByteArray::new_from_bytes(
            b"EIZ2\x05\xf7q\xc7G\x96\x1f\xba0\xe2\xd1\xf5pE\x14\xd7?\xac\xff\x8d\x1a\x0c\x11\x900f5\xfb\xff4\x94\xb8@\xc5^\xc2,exn0\xe3\xf0\n"
        );

        let first_operation = ManagedBuffer::from("first_operation");
        let first_operation_hash = self.get_operation_hash(first_operation);
        let second_operation = ManagedBuffer::from("second_operation");
        let second_operation_hash = self.get_operation_hash(second_operation);

        let mut bridge_operations: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>> =
            MultiValueEncoded::new();
        bridge_operations.push(first_operation_hash.clone());
        bridge_operations.push(second_operation_hash.clone());

        let mut bridge_operation = first_operation_hash;
        bridge_operation.append(&second_operation_hash);

        let bridge_operations_hash = self.get_operation_hash(bridge_operation);

        BridgeOperation {
            signature: mock_signature,
            bridge_operation_hash: bridge_operations_hash,
            operations_hashes: bridge_operations,
        }
    }

    fn get_operation_hash(
        &mut self,
        operation: ManagedBuffer<StaticApi>,
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

    let operation = state.get_mock_operation();
    state.propose_register_operations(operation);

    state
        .world
        .query()
        .to(HEADER_VERIFIER_ADDRESS)
        .whitebox(header_verifier::contract_obj, |sc| {
            assert!(!sc.hash_of_hashes_history().is_empty());
        })
}
