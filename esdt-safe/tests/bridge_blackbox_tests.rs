use esdt_safe::esdt_safe_proxy;
use multiversx_sc::{imports::MultiValueVec, 
    types::{
        TestAddress, TestSCAddress
    }}
;
use multiversx_sc_scenario::{
    imports::MxscPath, ScenarioTxRun, ScenarioWorld
};

const BRIDGE_ADDRESS: TestSCAddress = TestSCAddress::new("bridge");
const BRIDGE_CODE_PATH: MxscPath = MxscPath::new("output/esdt-safe.mxsc.json");
// const HEADER_VERIFIER_ADDRESS: TestSCAddress = TestSCAddress::new("header_verifier");
const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");
const USER_ADDRESS: TestAddress = TestAddress::new("user");
const OWNER_BALANCE: u64 = 100_000_000;
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
            .account(OWNER_ADDRESS)
            .nonce(1)
            .balance(OWNER_BALANCE)
            .account(USER_ADDRESS)
            .nonce(1)
            .balance(USER_BALANCE);

        Self { world }
    }

    fn deploy_bridge_contract(&mut self, is_sovereign_chain: bool) -> &mut Self {
        let signers = MultiValueVec::from(vec![USER_ADDRESS]);
        
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .init(is_sovereign_chain, 1u32, OWNER_ADDRESS, signers)
            .code(BRIDGE_CODE_PATH)
            .new_address(BRIDGE_ADDRESS)
            .run();

        self
    }
}

#[test]
fn test_deploy() {
    let mut state = BridgeTestState::new();

    state.deploy_bridge_contract(false);
}
