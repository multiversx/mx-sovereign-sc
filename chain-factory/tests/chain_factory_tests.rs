use chain_factory::chain_factory_proxy;
use multiversx_sc::types::{TestAddress, TestSCAddress};
use multiversx_sc_scenario::{imports::MxscPath, managed_biguint, ScenarioTxRun, ScenarioWorld};

const FACTORY_ADDRESS: TestSCAddress = TestSCAddress::new("chain-factory");
const CODE_PATH: MxscPath = MxscPath::new("output/chain-factory.mxsc.json");
const OWNER: TestAddress = TestAddress::new("owner");
const DEPLOY_COST: u64 = 100_000_000_000;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(CODE_PATH, chain_factory::ContractBuilder);

    blockchain
}

struct ChainFactoryTestState {
    world: ScenarioWorld,
}

impl ChainFactoryTestState {
    fn new() -> Self {
        let mut world = world();

        world.account(OWNER).balance(100_000).nonce(1);

        Self { world }
    }

    fn deploy_chain_factory(&mut self) {
        self.world
            .tx()
            .from(OWNER)
            .typed(chain_factory_proxy::ChainFactoryContractProxy)
            .init(
                FACTORY_ADDRESS,
                FACTORY_ADDRESS,
                FACTORY_ADDRESS,
                FACTORY_ADDRESS,
                FACTORY_ADDRESS,
                FACTORY_ADDRESS,
                managed_biguint!(DEPLOY_COST),
            )
            .code(CODE_PATH)
            .new_address(FACTORY_ADDRESS)
            .run();
    }
}

#[test]
fn deploy_test() {
    let mut state = ChainFactoryTestState::new();

    state.deploy_chain_factory();
}
