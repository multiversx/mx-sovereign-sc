use multiversx_sc::types::{ManagedBuffer, TestAddress, TestSCAddress};
use multiversx_sc_scenario::ScenarioTxRun;
use multiversx_sc_scenario::{api::StaticApi, imports::MxscPath, ScenarioWorld};
use token_handler::token_handler_proxy;

const TOKEN_HANDLER_ADDRESS: TestSCAddress = TestSCAddress::new("token-handler");
const TOKEN_HANDLER_CODE_PATH: MxscPath = MxscPath::new("output/token-handler.mxsc.json");
const OWNER_ADDRESS: TestAddress = TestAddress::new("token-handler-owner");

const OWNER_BALANCE: u64 = 100_000_000;
const CHAIN_PREFIX: &str = "sov";

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(TOKEN_HANDLER_CODE_PATH, token_handler::ContractBuilder);

    blockchain
}

struct TokenHandlerTestState {
    world: ScenarioWorld,
}

impl TokenHandlerTestState {
    fn new() -> Self {
        let mut world = world();

        world.account(OWNER_ADDRESS).nonce(1).balance(OWNER_BALANCE);

        Self { world }
    }

    fn propose_deploy(&mut self, chain_prefix: ManagedBuffer<StaticApi>) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(token_handler_proxy::TokenHandlerProxy)
            .init(chain_prefix)
            .code(TOKEN_HANDLER_CODE_PATH)
            .new_address(TOKEN_HANDLER_ADDRESS)
            .run();

        self
    }
}

#[test]
fn test_deploy() {
    let mut state = TokenHandlerTestState::new();

    state.propose_deploy(CHAIN_PREFIX.into());
}
