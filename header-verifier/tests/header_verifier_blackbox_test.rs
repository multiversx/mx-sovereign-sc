use header_verifier::{header_verifier_proxy, Headerverifier};
use multiversx_sc::types::{BigUint, ManagedBuffer, MultiValueEncoded, TestAddress, TestSCAddress};
use multiversx_sc_scenario::{
    api::StaticApi, imports::MxscPath, ScenarioTxRun, ScenarioTxWhitebox, ScenarioWorld,
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

    fn get_bls_keys(&mut self, bls_keys_vec: Vec<ManagedBuffer<StaticApi>>) -> BlsKeys {
        let bls_keys = bls_keys_vec.iter().map(|key| key.clone().into()).collect();

        bls_keys
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
