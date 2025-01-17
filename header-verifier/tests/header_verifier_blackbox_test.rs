use chain_config::validator_rules::ValidatorRulesModule;
use header_verifier::{Headerverifier, OperationHashStatus};
use multiversx_sc::types::ManagedBuffer;
use multiversx_sc::{
    api::ManagedTypeApi,
    types::{BigUint, MultiValueEncoded, TestAddress, TestSCAddress},
};
use multiversx_sc_scenario::ReturnsHandledOrError;
use multiversx_sc_scenario::{
    api::StaticApi, imports::MxscPath, multiversx_chain_vm::crypto_functions::sha256, DebugApi,
    ScenarioTxRun, ScenarioTxWhitebox, ScenarioWorld,
};
use operation::SovereignConfig;
use proxies::chain_config_proxy::ChainConfigContractProxy;
use proxies::header_verifier_proxy::HeaderverifierProxy;

const HEADER_VERIFIER_CODE_PATH: MxscPath = MxscPath::new("ouput/header-verifier.mxsc-json");
const HEADER_VERIFIER_ADDRESS: TestSCAddress = TestSCAddress::new("header-verifier");

const CHAIN_CONFIG_CODE_PATH: MxscPath =
    MxscPath::new("../chain-config/output/chain-config.mxsc-json");
const CHAIN_CONFIG_ADDRESS: TestSCAddress = TestSCAddress::new("chain-config");

// NOTE: This is a mock path
const ENSHRINE_ADDRESS: TestAddress = TestAddress::new("enshrine");
const DUMMY_SC_ADDRESS: TestSCAddress = TestSCAddress::new("dummy-sc");

const OWNER: TestAddress = TestAddress::new("owner");
const WEGLD_BALANCE: u128 = 100_000_000_000_000_000;

type BlsKeys = MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>>;

#[derive(Clone)]
pub struct BridgeOperation<M: ManagedTypeApi> {
    signature: ManagedBuffer<M>,
    bridge_operation_hash: ManagedBuffer<M>,
    operations_hashes: MultiValueEncoded<M, ManagedBuffer<M>>,
}

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(HEADER_VERIFIER_CODE_PATH, header_verifier::ContractBuilder);
    blockchain.register_contract(CHAIN_CONFIG_CODE_PATH, chain_config::ContractBuilder);

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

    fn deploy_header_verifier_contract(
        &mut self,
        chain_config_address: TestSCAddress,
        bls_keys: BlsKeys,
    ) -> &mut Self {
        self.world
            .tx()
            .from(OWNER)
            .typed(HeaderverifierProxy)
            .init(chain_config_address, bls_keys)
            .code(HEADER_VERIFIER_CODE_PATH)
            .new_address(HEADER_VERIFIER_ADDRESS)
            .run();

        self
    }

    fn deploy_chain_config(
        &mut self,
        sovereign_config: &SovereignConfig<StaticApi>,
        admin: TestSCAddress,
    ) -> &mut Self {
        self.world
            .tx()
            .from(OWNER)
            .typed(ChainConfigContractProxy)
            .init(sovereign_config, admin)
            .code(CHAIN_CONFIG_CODE_PATH)
            .new_address(CHAIN_CONFIG_ADDRESS)
            .run();

        self
    }

    fn propose_register_esdt_address(&mut self, esdt_address: TestAddress) {
        self.world
            .tx()
            .from(OWNER)
            .to(HEADER_VERIFIER_ADDRESS)
            .typed(HeaderverifierProxy)
            .set_esdt_safe_address(esdt_address)
            .run();
    }

    fn propose_register_operations(&mut self, operation: BridgeOperation<StaticApi>) {
        self.world
            .tx()
            .from(OWNER)
            .to(HEADER_VERIFIER_ADDRESS)
            .typed(HeaderverifierProxy)
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
        operation_hash: &ManagedBuffer<StaticApi>,
        error_message: Option<&str>,
    ) {
        let response = self
            .world
            .tx()
            .from(caller)
            .to(HEADER_VERIFIER_ADDRESS)
            .typed(HeaderverifierProxy)
            .remove_executed_hash(hash_of_hashes, operation_hash)
            .returns(ReturnsHandledOrError::new())
            .run();

        if let Err(error) = response {
            assert_eq!(error_message, Some(error.message.as_str()))
        }
    }

    fn propose_lock_operation_hash(
        &mut self,
        caller: TestAddress,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        operation_hash: &ManagedBuffer<StaticApi>,
        error_message: Option<&str>,
    ) {
        let response = self
            .world
            .tx()
            .from(caller)
            .to(HEADER_VERIFIER_ADDRESS)
            .typed(HeaderverifierProxy)
            .lock_operation_hash(hash_of_hashes, operation_hash)
            .returns(ReturnsHandledOrError::new())
            .run();

        if let Err(error) = response {
            assert_eq!(error_message, Some(error.message.as_str()))
        }
    }

    fn update_config(
        &mut self,
        new_config: SovereignConfig<StaticApi>,
        error_message: Option<&str>,
    ) {
        let response = self
            .world
            .tx()
            .from(OWNER)
            .to(HEADER_VERIFIER_ADDRESS)
            .typed(HeaderverifierProxy)
            .update_config(new_config)
            .returns(ReturnsHandledOrError::new())
            .run();

        if let Err(error) = response {
            assert_eq!(error_message, Some(error.message.as_str()))
        }
    }

    fn change_validator_set(&mut self, bls_keys: BlsKeys, error_message: Option<&str>) {
        let response = self
            .world
            .tx()
            .from(OWNER)
            .to(HEADER_VERIFIER_ADDRESS)
            .typed(HeaderverifierProxy)
            .change_validator_set(bls_keys)
            .returns(ReturnsHandledOrError::new())
            .run();

        if let Err(error) = response {
            assert_eq!(error_message, Some(error.message.as_str()))
        }
    }

    fn get_bls_keys(&mut self, bls_keys_vec: Vec<ManagedBuffer<StaticApi>>) -> BlsKeys {
        let bls_keys = bls_keys_vec.iter().cloned().collect();

        bls_keys
    }

    fn generate_bridge_operation_struct(
        &mut self,
        operation_hashes: Vec<&ManagedBuffer<StaticApi>>,
    ) -> BridgeOperation<StaticApi> {
        let mock_signature = ManagedBuffer::new();

        let mut bridge_operations: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>> =
            MultiValueEncoded::new();
        let mut appended_hashes = ManagedBuffer::new();

        for operation_hash in operation_hashes {
            appended_hashes.append(operation_hash);
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

    state.deploy_header_verifier_contract(CHAIN_CONFIG_ADDRESS, managed_bls_keys);
}

#[test]
fn test_register_esdt_address() {
    let mut state = HeaderVerifierTestState::new();
    let bls_key_1 = ManagedBuffer::from("bls_key_1");
    let managed_bls_keys = state.get_bls_keys(vec![bls_key_1]);

    state.deploy_header_verifier_contract(CHAIN_CONFIG_ADDRESS, managed_bls_keys);

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

    state.deploy_header_verifier_contract(CHAIN_CONFIG_ADDRESS, managed_bls_keys);

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

            for operation_hash in operation.operations_hashes {
                let operation_hash_debug_api = ManagedBuffer::from(operation_hash.to_vec());

                let pending_hashes_mapper =
                    sc.operation_hash_status(&hash_of_hashes, &operation_hash_debug_api);

                let is_mapper_empty = pending_hashes_mapper.is_empty();
                let is_operation_hash_locked = pending_hashes_mapper.get();

                assert!(!is_mapper_empty);
                assert!(is_operation_hash_locked == OperationHashStatus::NotLocked);
            }
        });
}

#[test]
fn test_remove_executed_hash_caller_not_esdt_address() {
    let mut state = HeaderVerifierTestState::new();
    let bls_key_1 = ManagedBuffer::from("bls_key_1");
    let managed_bls_keys = state.get_bls_keys(vec![bls_key_1]);

    state.deploy_header_verifier_contract(CHAIN_CONFIG_ADDRESS, managed_bls_keys);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);

    state.propose_register_operations(operation.clone());
    state.propose_register_esdt_address(ENSHRINE_ADDRESS);
    state.propose_remove_executed_hash(
        OWNER,
        &operation.bridge_operation_hash,
        &operation_1,
        Some("Only ESDT Safe contract can call this endpoint"),
    );
}

#[test]
fn test_remove_executed_hash_no_esdt_address_registered() {
    let mut state = HeaderVerifierTestState::new();
    let bls_key_1 = ManagedBuffer::from("bls_key_1");
    let managed_bls_keys = state.get_bls_keys(vec![bls_key_1]);

    state.deploy_header_verifier_contract(CHAIN_CONFIG_ADDRESS, managed_bls_keys);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);

    state.propose_register_operations(operation.clone());
    state.propose_remove_executed_hash(
        ENSHRINE_ADDRESS,
        &operation.bridge_operation_hash,
        &operation_1,
        Some("There is no registered ESDT address"),
    );
}

#[test]
fn test_remove_one_executed_hash() {
    let mut state = HeaderVerifierTestState::new();
    let bls_key_1 = ManagedBuffer::from("bls_key_1");
    let managed_bls_keys = state.get_bls_keys(vec![bls_key_1]);

    state.deploy_header_verifier_contract(CHAIN_CONFIG_ADDRESS, managed_bls_keys);

    let operation_hash_1 = ManagedBuffer::from("operation_1");
    let operation_hash_2 = ManagedBuffer::from("operation_2");
    let operation =
        state.generate_bridge_operation_struct(vec![&operation_hash_1, &operation_hash_2]);

    state.propose_register_operations(operation.clone());
    state.propose_register_esdt_address(ENSHRINE_ADDRESS);

    state.propose_remove_executed_hash(
        ENSHRINE_ADDRESS,
        &operation.bridge_operation_hash,
        &operation_hash_1,
        None,
    );

    state
        .world
        .query()
        .to(HEADER_VERIFIER_ADDRESS)
        .whitebox(header_verifier::contract_obj, |sc| {
            let hash_of_hashes: ManagedBuffer<DebugApi> =
                ManagedBuffer::from(operation.bridge_operation_hash.to_vec());
            let operation_hash_debug_api = ManagedBuffer::from(operation_hash_2.to_vec());

            let pending_hashes_mapper =
                sc.operation_hash_status(&hash_of_hashes, &operation_hash_debug_api);

            let is_hash_locked = pending_hashes_mapper.get();
            let is_mapper_empty = pending_hashes_mapper.is_empty();

            assert!(!is_mapper_empty);
            assert!(is_hash_locked == OperationHashStatus::NotLocked);
        });
}

#[test]
fn test_remove_all_executed_hashes() {
    let mut state = HeaderVerifierTestState::new();
    let bls_key_1 = ManagedBuffer::from("bls_key_1");
    let managed_bls_keys = state.get_bls_keys(vec![bls_key_1]);

    state.deploy_header_verifier_contract(CHAIN_CONFIG_ADDRESS, managed_bls_keys);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);

    state.propose_register_operations(operation.clone());
    state.propose_register_esdt_address(ENSHRINE_ADDRESS);

    state.propose_remove_executed_hash(
        ENSHRINE_ADDRESS,
        &operation.bridge_operation_hash,
        &operation_1,
        None,
    );

    state.propose_remove_executed_hash(
        ENSHRINE_ADDRESS,
        &operation.bridge_operation_hash,
        &operation_2,
        None,
    );
    state
        .world
        .query()
        .to(HEADER_VERIFIER_ADDRESS)
        .whitebox(header_verifier::contract_obj, |sc| {
            let hash_of_hashes: ManagedBuffer<DebugApi> =
                ManagedBuffer::from(operation.bridge_operation_hash.to_vec());
            let operation_hash_debug_api_1 = ManagedBuffer::from(operation_1.to_vec());
            let operation_hash_debug_api_2 = ManagedBuffer::from(operation_2.to_vec());
            assert!(sc
                .operation_hash_status(&hash_of_hashes, &operation_hash_debug_api_1)
                .is_empty());
            assert!(sc
                .operation_hash_status(&hash_of_hashes, &operation_hash_debug_api_2)
                .is_empty());
            assert!(sc.hash_of_hashes_history().contains(&hash_of_hashes));
        });
}

#[test]
fn test_lock_operation_not_registered() {
    let mut state = HeaderVerifierTestState::new();
    let bls_key_1 = ManagedBuffer::from("bls_key_1");
    let managed_bls_keys = state.get_bls_keys(vec![bls_key_1]);

    state.deploy_header_verifier_contract(CHAIN_CONFIG_ADDRESS, managed_bls_keys);
    state.propose_register_esdt_address(ENSHRINE_ADDRESS);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);

    state.propose_lock_operation_hash(
        ENSHRINE_ADDRESS,
        &operation.bridge_operation_hash,
        &operation_1,
        Some("The current operation is not registered"),
    );
}

#[test]
fn test_lock_operation() {
    let mut state = HeaderVerifierTestState::new();
    let bls_key_1 = ManagedBuffer::from("bls_key_1");
    let managed_bls_keys = state.get_bls_keys(vec![bls_key_1]);

    state.deploy_header_verifier_contract(CHAIN_CONFIG_ADDRESS, managed_bls_keys);
    state.propose_register_esdt_address(ENSHRINE_ADDRESS);

    let operation_1 = ManagedBuffer::from("operation_1");
    let operation_2 = ManagedBuffer::from("operation_2");
    let operation = state.generate_bridge_operation_struct(vec![&operation_1, &operation_2]);

    state.propose_register_operations(operation.clone());

    state.propose_lock_operation_hash(
        ENSHRINE_ADDRESS,
        &operation.bridge_operation_hash,
        &operation_1,
        None,
    );

    state
        .world
        .query()
        .to(HEADER_VERIFIER_ADDRESS)
        .whitebox(header_verifier::contract_obj, |sc| {
            let hash_of_hashes: ManagedBuffer<DebugApi> =
                ManagedBuffer::from(operation.bridge_operation_hash.to_vec());
            let operation_hash_debug_api_1 = ManagedBuffer::from(operation_1.to_vec());
            let operation_hash_debug_api_2 = ManagedBuffer::from(operation_2.to_vec());
            let is_hash_1_locked = sc
                .operation_hash_status(&hash_of_hashes, &operation_hash_debug_api_1)
                .get();
            let is_hash_2_locked = sc
                .operation_hash_status(&hash_of_hashes, &operation_hash_debug_api_2)
                .get();

            assert!(is_hash_1_locked == OperationHashStatus::Locked);
            assert!(is_hash_2_locked == OperationHashStatus::NotLocked);
        })
}

#[test]
fn update_config_can_only_be_called_by_chain_config_admin() {
    let mut state = HeaderVerifierTestState::new();
    let bls_key_1 = ManagedBuffer::from("bls_key_1");
    let managed_bls_keys = state.get_bls_keys(vec![bls_key_1]);

    state.deploy_header_verifier_contract(CHAIN_CONFIG_ADDRESS, managed_bls_keys);

    let sovereign_config = SovereignConfig::new(0, 0, BigUint::default(), None);

    state.deploy_chain_config(&sovereign_config, DUMMY_SC_ADDRESS);
    state.update_config(
        sovereign_config,
        Some("Endpoint can only be called by admins"),
    );
}

#[test]
fn update_config_wrong_validator_range() {
    let mut state = HeaderVerifierTestState::new();
    let bls_key_1 = ManagedBuffer::from("bls_key_1");
    let managed_bls_keys = state.get_bls_keys(vec![bls_key_1]);

    state.deploy_header_verifier_contract(CHAIN_CONFIG_ADDRESS, managed_bls_keys);

    let sovereign_config = SovereignConfig::new(0, 0, BigUint::default(), None);

    state.deploy_chain_config(&sovereign_config, DUMMY_SC_ADDRESS);

    let new_config = SovereignConfig::new(1, 0, BigUint::default(), None);
    state.update_config(
        new_config,
        Some("The min_validators number should lower or equal to the number of max_validators"),
    );
}

#[test]
fn update_config() {
    let mut state = HeaderVerifierTestState::new();
    let bls_key_1 = ManagedBuffer::from("bls_key_1");
    let managed_bls_keys = state.get_bls_keys(vec![bls_key_1]);

    state.deploy_header_verifier_contract(CHAIN_CONFIG_ADDRESS, managed_bls_keys);

    let sovereign_config = SovereignConfig::new(0, 0, BigUint::default(), None);

    state.deploy_chain_config(&sovereign_config, HEADER_VERIFIER_ADDRESS);
    state.update_config(sovereign_config, None);

    state
        .world
        .query()
        .to(CHAIN_CONFIG_ADDRESS)
        .whitebox(chain_config::contract_obj, |sc| {
            let sovereign_config_mapper = sc.sovereign_config();

            assert!(!sovereign_config_mapper.is_empty());
            let sovereign_config: SovereignConfig<DebugApi> =
                SovereignConfig::new(0, 0, BigUint::default(), None);

            let stored_sovereign_config = sovereign_config_mapper.get();

            assert!(sovereign_config == stored_sovereign_config);
        })
}

#[test]
fn change_validator_set_incorect_length() {
    let mut state = HeaderVerifierTestState::new();
    let bls_key_1 = ManagedBuffer::from("bls_key_1");
    let managed_bls_keys = state.get_bls_keys(vec![bls_key_1]);

    state.deploy_header_verifier_contract(CHAIN_CONFIG_ADDRESS, managed_bls_keys);

    let sovereign_config = SovereignConfig::new(0, 0, BigUint::default(), None);

    state.deploy_chain_config(&sovereign_config, HEADER_VERIFIER_ADDRESS);

    let new_validator_set = state.get_bls_keys(vec![ManagedBuffer::from("some_other_bls_key")]);
    state.change_validator_set(
        new_validator_set,
        Some("The current validator set lenght doesn't meet the Sovereign's requirements"),
    );
}

#[test]
fn change_validator_set() {
    let mut state = HeaderVerifierTestState::new();
    let bls_key_1 = ManagedBuffer::from("bls_key_1");
    let managed_bls_keys = state.get_bls_keys(vec![bls_key_1]);

    state.deploy_header_verifier_contract(CHAIN_CONFIG_ADDRESS, managed_bls_keys);

    let sovereign_config = SovereignConfig::new(1, 2, BigUint::default(), None);

    state.deploy_chain_config(&sovereign_config, HEADER_VERIFIER_ADDRESS);

    let new_validator_set = state.get_bls_keys(vec![
        ManagedBuffer::from("new_key_1"),
        ManagedBuffer::from("new_key_2"),
    ]);
    state.change_validator_set(new_validator_set, None);

    state
        .world
        .query()
        .to(HEADER_VERIFIER_ADDRESS)
        .whitebox(header_verifier::contract_obj, |sc| {
            assert!(!sc.bls_pub_keys().is_empty());

            let bls_key_1 = ManagedBuffer::from("new_key_1");
            let bls_key_2 = ManagedBuffer::from("new_key_2");

            assert!(sc.bls_pub_keys().contains(&bls_key_1));
            assert!(sc.bls_pub_keys().contains(&bls_key_2));
        })
}
