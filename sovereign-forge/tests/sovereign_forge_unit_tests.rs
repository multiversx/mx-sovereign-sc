use multiversx_sc::types::{BigUint, TestAddress, TestSCAddress};
use multiversx_sc_scenario::{
    imports::MxscPath, ExpectError, ScenarioTxRun, ScenarioTxWhitebox, ScenarioWorld,
};
use proxies::{
    chain_factory_proxy::ChainFactoryContractProxy, sovereign_forge_proxy::SovereignForgeProxy,
};
use setup_phase::SetupPhaseModule;
use sovereign_forge::common::storage::StorageModule;

const FORGE_ADDRESS: TestSCAddress = TestSCAddress::new("sovereign-forge");
const FORGE_CODE_PATH: MxscPath = MxscPath::new("output/sovereign-forge.mxsc.json");
const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");

const FACTORY_ADDRESS: TestSCAddress = TestSCAddress::new("chain-factory");
const FACTORY_CODE_PATH: MxscPath =
    MxscPath::new("../chain-factory/output/chain-factory.mxsc.json");

const TOKEN_HANDLER_ADDRESS: TestSCAddress = TestSCAddress::new("token-handler");

const BALANCE: u128 = 100_000_000_000_000_000;
const DEPLOY_COST: u64 = 100_000;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(FORGE_CODE_PATH, sovereign_forge::ContractBuilder);
    blockchain.register_contract(FACTORY_CODE_PATH, chain_factory::ContractBuilder);

    blockchain
}

struct SovereignForgeTestState {
    world: ScenarioWorld,
}

impl SovereignForgeTestState {
    fn new() -> Self {
        let mut world = world();

        world
            .account(OWNER_ADDRESS)
            .balance(BigUint::from(BALANCE))
            .nonce(1);

        Self { world }
    }

    fn deploy_chain_factory(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(ChainFactoryContractProxy)
            .init(
                FACTORY_ADDRESS,
                FACTORY_ADDRESS,
                FACTORY_ADDRESS,
                FACTORY_ADDRESS,
            )
            .code(FACTORY_CODE_PATH)
            .new_address(FACTORY_ADDRESS)
            .run();

        self
    }

    fn deploy_sovereign_forge(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(SovereignForgeProxy)
            .init(DEPLOY_COST)
            .code(FORGE_CODE_PATH)
            .new_address(FORGE_ADDRESS)
            .run();

        self
    }

    fn register_token_handler(
        &mut self,
        shard_id: u32,
        token_handler_address: TestSCAddress,
        expected_result: Option<ExpectError>,
    ) {
        let transaction = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(FORGE_ADDRESS)
            .typed(SovereignForgeProxy)
            .register_token_handler(shard_id, token_handler_address);

        if let Some(error) = expected_result {
            transaction.returns(error).run();
        } else {
            transaction.run();
        }
    }

    fn register_chain_factory(
        &mut self,
        shard_id: u32,
        chain_factory_address: TestSCAddress,
        expected_result: Option<ExpectError>,
    ) {
        let transaction = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(FORGE_ADDRESS)
            .typed(SovereignForgeProxy)
            .register_chain_factory(shard_id, chain_factory_address);

        if let Some(error) = expected_result {
            transaction.returns(error).run();
        } else {
            transaction.run();
        }
    }

    fn complete_setup_phase(&mut self, expected_result: Option<ExpectError>) {
        let transaction = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(FORGE_ADDRESS)
            .typed(SovereignForgeProxy)
            .complete_setup_phase();

        if let Some(error) = expected_result {
            transaction.returns(error).run();
        } else {
            transaction.run();
        }
    }
}

#[test]
fn test_deploy_forge() {
    let mut state = SovereignForgeTestState::new();
    state.deploy_sovereign_forge();
}

#[test]
fn test_chain_factory() {
    let mut state = SovereignForgeTestState::new();
    state.deploy_chain_factory();
}

#[test]
fn test_register_token_handler() {
    let mut state = SovereignForgeTestState::new();
    state.deploy_sovereign_forge();

    state.register_token_handler(2, FACTORY_ADDRESS, None);

    state
        .world
        .query()
        .to(FORGE_ADDRESS)
        .whitebox(sovereign_forge::contract_obj, |sc| {
            assert!(!sc.token_handlers(2).is_empty());
        });
}

#[test]
fn test_register_chain_factory() {
    let mut state = SovereignForgeTestState::new();
    state.deploy_sovereign_forge();

    state.register_chain_factory(2, TOKEN_HANDLER_ADDRESS, None);

    state
        .world
        .query()
        .to(FORGE_ADDRESS)
        .whitebox(sovereign_forge::contract_obj, |sc| {
            assert!(!sc.chain_factories(2).is_empty());
        });
}

#[test]
fn complete_setup_phase_no_chain_config_registered() {
    let mut state = SovereignForgeTestState::new();
    state.deploy_sovereign_forge();

    state.complete_setup_phase(Some(ExpectError(
        4,
        "There is no Chain-Factory contract assigned for shard 1",
    )));
}

#[test]
fn complete_setup_phase_no_token_handler_registered() {
    let mut state = SovereignForgeTestState::new();
    state.deploy_sovereign_forge();
    state.register_chain_factory(1, FACTORY_ADDRESS, None);

    state.complete_setup_phase(Some(ExpectError(
        4,
        "There is no Token-Handler contract assigned for shard 1",
    )));
}

#[test]
fn complete_setup_phase() {
    let mut state = SovereignForgeTestState::new();
    state.deploy_sovereign_forge();
    state.register_chain_factory(1, FACTORY_ADDRESS, None);
    state.register_chain_factory(2, FACTORY_ADDRESS, None);
    state.register_chain_factory(3, FACTORY_ADDRESS, None);
    state.register_token_handler(1, TOKEN_HANDLER_ADDRESS, None);
    state.register_token_handler(2, TOKEN_HANDLER_ADDRESS, None);
    state.register_token_handler(3, TOKEN_HANDLER_ADDRESS, None);

    state.complete_setup_phase(None);

    state
        .world
        .query()
        .to(FORGE_ADDRESS)
        .whitebox(sovereign_forge::contract_obj, |sc| {
            assert!(sc.is_setup_phase_complete());
        });
}
