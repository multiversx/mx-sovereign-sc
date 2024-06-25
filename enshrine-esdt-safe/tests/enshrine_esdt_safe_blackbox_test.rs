use enshrine_esdt_safe::enshrine_esdt_safe_proxy;
use multiversx_sc::types::{ManagedBuffer, TestAddress, TestSCAddress, TestTokenIdentifier};
use multiversx_sc_scenario::ScenarioTxRun;
use multiversx_sc_scenario::{imports::MxscPath, ScenarioWorld};

const ENSHRINE_ESDT_ADDRESS: TestSCAddress = TestSCAddress::new("enshrine-esdt");
const ENSHRINE_ESDT_CODE_PATH: MxscPath = MxscPath::new("output/enshrine-esdt-safe.mxsc-json");
const ENSHRINE_ESDT_OWNER: TestAddress = TestAddress::new("enshrine-esdt-owner");

const ENSHRINE_OWNER_BALANCE: u64 = 100_000_000;
const USER_EGLD_BALANCE: u64 = 100_000_000;

const NFT_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("NFT-123456");
const FUNGIBLE_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("CROWD-123456");

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(ENSHRINE_ESDT_CODE_PATH, enshrine_esdt_safe::ContractBuilder);

    blockchain
}

struct EnshrineTestState {
    world: ScenarioWorld,
}

impl EnshrineTestState {
    fn new() -> Self {
        let mut world = world();

        world
            .account(ENSHRINE_ESDT_OWNER)
            .esdt_balance(FUNGIBLE_TOKEN_ID, 100_000)
            .esdt_nft_balance(NFT_TOKEN_ID, 1, 100_000, ManagedBuffer::new())
            .nonce(1)
            .balance(ENSHRINE_OWNER_BALANCE);

        Self { world }
    }

    fn deploy_enshrine_esdt_contract(&mut self, is_sovereign_chain: bool) -> &mut Self {
        self.world
            .tx()
            .from(ENSHRINE_ESDT_OWNER)
            .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
            .init(is_sovereign_chain)
            .code(ENSHRINE_ESDT_CODE_PATH)
            .new_address(ENSHRINE_ESDT_ADDRESS)
            .run();

        self
    }
}

#[test]
fn test_deploy() {
    let mut state = EnshrineTestState::new();

    state.deploy_enshrine_esdt_contract(false);
}
