use esdt_safe::{endpoints::unpause_endpoint, esdt_safe_proxy};
use multiversx_sc::{
    imports::{MultiValue3, MultiValueVec, OptionalValue},
    types::{ManagedBuffer, ManagedVec, ReturnsResult, TestAddress, TestSCAddress, Tx},
};
use multiversx_sc_scenario::{
    api::StaticApi, imports::MxscPath, ExpectError, ScenarioTxEnv, ScenarioTxRun, ScenarioWorld
};

const BRIDGE_ADDRESS: TestSCAddress = TestSCAddress::new("bridge");
const BRIDGE_CODE_PATH: MxscPath = MxscPath::new("output/esdt-safe.mxsc.json");
const BRIDGE_OWNER_ADDRESS: TestAddress = TestAddress::new("bridge_owner");

const USER_ADDRESS: TestAddress = TestAddress::new("user");
const RECEIVER_ADDRESS: TestAddress = TestAddress::new("receiver");

const BRIDGE_OWNER_BALANCE: u64 = 100_000_000;
const USER_BALANCE: u64 = 100_000_000;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.set_current_dir_from_workspace("mx-sovereign-sc/esdt-safe");
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
            .account(RECEIVER_ADDRESS)
            .nonce(1);

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

    fn propose_egld_deposit_and_expect_err(&mut self, err_message: &str) {
        let transfer_data = OptionalValue::<
            MultiValue3<
                u64,
                ManagedBuffer<StaticApi>,
                ManagedVec<StaticApi, ManagedBuffer<StaticApi>>,
            >,
        >::None;

        self.world
            .tx()
            .from(USER_ADDRESS)
            .to(BRIDGE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .deposit(RECEIVER_ADDRESS, transfer_data)
            .egld(10)
            .with_result(ExpectError(4, err_message))
            .run();
    }

    fn propose_set_unpaused(&mut self) {
        self
            .world
            .tx()
            .from(BRIDGE_OWNER_ADDRESS)
            .to(BRIDGE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .unpause_endpoint()
            .returns(ReturnsResult)
            .run();
    }
}

#[test]
fn test_deploy() {
    let mut state = BridgeTestState::new();

    state.deploy_bridge_contract(false);
}

#[test]
fn test_egld_deposit_nothing_to_transfer() {
    let mut state = BridgeTestState::new();
    let err_message = "Nothing to transfer";

    state.deploy_bridge_contract(false);
    state.propose_set_unpaused();
    state.propose_egld_deposit_and_expect_err(err_message);
}
