use esdt_safe::esdt_safe_proxy;
use header_verifier::header_verifier_proxy;
use multiversx_sc::{
    abi::{TypeAbi, TypeAbiFrom},
    imports::{MultiValue3, MultiValueVec, OptionalValue},
    types::{ManagedBuffer, TestAddress, TestSCAddress},
};
use multiversx_sc_scenario::{api::StaticApi, imports::MxscPath, ScenarioTxRun, ScenarioWorld};
use transaction::GasLimit;

const BRIDGE_ADDRESS: TestSCAddress = TestSCAddress::new("bridge");
const BRIDGE_CODE_PATH: MxscPath = MxscPath::new("output/esdt-safe.mxsc.json");
const BRIDGE_OWNER_ADDRESS: TestAddress = TestAddress::new("bridge_owner");

const HEADER_VERIFIER_ADDRESS: TestSCAddress = TestSCAddress::new("header_verifier");
const HEADER_VERIFIER_CODE_PATH: MxscPath =
    MxscPath::new("../../header-verifier/output/header-verifier.mxsc.json");
const HEADER_OWNER_ADDRESS: TestAddress = TestAddress::new("header_owner");

const USER_ADDRESS: TestAddress = TestAddress::new("user");
const RECEIVER_ADDRESS: TestAddress = TestAddress::new("receiver");

const BRIDGE_OWNER_BALANCE: u64 = 100_000_000;
const HEADER_OWNER_BALANCE: u64 = 100_000_000;
const USER_BALANCE: u64 = 100_000_000;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.set_current_dir_from_workspace("esdt-safe");
    blockchain.register_contract(BRIDGE_CODE_PATH, esdt_safe::ContractBuilder);

    blockchain
}

struct BridgeTestState {
    world: ScenarioWorld,
}

impl BridgeTestState {
    fn new() -> Self {
        let mut world = world();

        world
            .account(BRIDGE_OWNER_ADDRESS)
            .nonce(1)
            .balance(BRIDGE_OWNER_BALANCE)
            .account(USER_ADDRESS)
            .nonce(1)
            .balance(USER_BALANCE)
            .account(HEADER_OWNER_ADDRESS)
            .nonce(1)
            .balance(HEADER_OWNER_BALANCE);

        Self { world }
    }

    fn deploy_bridge_contract(&mut self, is_sovereign_chain: bool) -> &mut Self {
        let signers = MultiValueVec::from(vec![USER_ADDRESS]);

        self.world
            .tx()
            .from(BRIDGE_OWNER_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .init(is_sovereign_chain, 1u32, BRIDGE_OWNER_ADDRESS, signers)
            .code(BRIDGE_CODE_PATH)
            .new_address(BRIDGE_ADDRESS)
            .run();

        self
    }

    fn deploy_header_verifier_contract(&mut self) -> &mut Self {
        let bls_pub_keys = MultiValueVec::from(vec![ManagedBuffer::new()]);

        self.world
            .tx()
            .from(HEADER_OWNER_ADDRESS)
            .typed(header_verifier_proxy::HeaderverifierProxy)
            .init(bls_pub_keys)
            .code(HEADER_VERIFIER_CODE_PATH)
            .new_address(HEADER_VERIFIER_ADDRESS)
            .run();

        self
    }

    fn propose_set_header_verifier_address(&mut self) {
        self.world
            .tx()
            .from(BRIDGE_OWNER_ADDRESS)
            .to(BRIDGE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .set_header_verifier_address(HEADER_VERIFIER_ADDRESS)
            .run();
    }

    fn propose_egld_deposit(&mut self) {
        let transfer_data = OptionalValue::<
            MultiValue3<u64, ManagedBuffer<StaticApi>, MultiValueVec<ManagedBuffer<StaticApi>>>
        >::None;

        self.world
            .tx()
            .from(USER_ADDRESS)
            .to(BRIDGE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .deposit(RECEIVER_ADDRESS, transfer_data)
            .egld(10)
            .run();
    }
}

#[test]
fn test_deploy() {
    let mut state = BridgeTestState::new();

    state.deploy_bridge_contract(false);
    state.deploy_header_verifier_contract();
}
