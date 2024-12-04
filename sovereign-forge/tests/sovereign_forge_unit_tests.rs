use multiversx_sc::types::{BigUint, ManagedBuffer, MultiValueEncoded, TestAddress, TestSCAddress};
use multiversx_sc_scenario::{
    api::StaticApi, imports::MxscPath, ExpectError, ScenarioTxRun, ScenarioTxWhitebox,
    ScenarioWorld,
};
use proxies::{
    chain_config_proxy::ChainConfigContractProxy, chain_factory_proxy::ChainFactoryContractProxy,
    header_verifier_proxy::HeaderverifierProxy, sovereign_forge_proxy::SovereignForgeProxy,
};
use setup_phase::SetupPhaseModule;
use sovereign_forge::common::{storage::StorageModule, utils::ScArray};
use transaction::StakeMultiArg;

const FORGE_ADDRESS: TestSCAddress = TestSCAddress::new("sovereign-forge");
const FORGE_CODE_PATH: MxscPath = MxscPath::new("output/sovereign-forge.mxsc.json");
const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");

const FACTORY_ADDRESS: TestSCAddress = TestSCAddress::new("chain-factory");
const FACTORY_CODE_PATH: MxscPath =
    MxscPath::new("../chain-factory/output/chain-factory.mxsc.json");

const CONFIG_ADDRESS: TestSCAddress = TestSCAddress::new("chain-config");
const CONFIG_CODE_PATH: MxscPath = MxscPath::new("../chain-config/output/chain-config.mxsc.json");

const HEADER_VERIFIER_ADDRESS: TestSCAddress = TestSCAddress::new("header-verifier");
const HEADER_VERIFIER_CODE_PATH: MxscPath =
    MxscPath::new("../header-verifier/output/header-verifier.mxsc.json");

const TOKEN_HANDLER_ADDRESS: TestSCAddress = TestSCAddress::new("token-handler");

const BALANCE: u128 = 100_000_000_000_000_000;
const DEPLOY_COST: u64 = 100_000;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(FORGE_CODE_PATH, sovereign_forge::ContractBuilder);
    blockchain.register_contract(FACTORY_CODE_PATH, chain_factory::ContractBuilder);
    blockchain.register_contract(CONFIG_CODE_PATH, chain_config::ContractBuilder);
    blockchain.register_contract(HEADER_VERIFIER_CODE_PATH, header_verifier::ContractBuilder);

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
            .from(FORGE_ADDRESS)
            .typed(ChainFactoryContractProxy)
            .init(
                CONFIG_ADDRESS,
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

    fn deploy_chain_config_template(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(ChainConfigContractProxy)
            .init(
                1u64,
                2u64,
                BigUint::from(1u32),
                OWNER_ADDRESS,
                MultiValueEncoded::<StaticApi, StakeMultiArg<StaticApi>>::new(),
            )
            .code(CONFIG_CODE_PATH)
            .new_address(CONFIG_ADDRESS)
            .run();

        self
    }

    fn deploy_header_verifier_template(&mut self) -> &mut Self {
        let mut bls_keys = MultiValueEncoded::new();
        bls_keys.push(ManagedBuffer::from("bls1"));
        bls_keys.push(ManagedBuffer::from("bls2"));

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(HeaderverifierProxy)
            .init(bls_keys)
            .code(HEADER_VERIFIER_CODE_PATH)
            .new_address(HEADER_VERIFIER_ADDRESS)
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

    fn deploy_phase_one(
        &mut self,
        payment: BigUint<StaticApi>,
        min_validators: u64,
        max_validators: u64,
        min_stake: BigUint<StaticApi>,
        additional_stake_required: MultiValueEncoded<StaticApi, StakeMultiArg<StaticApi>>,
        expected_result: Option<ExpectError>,
    ) {
        let transaction = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(FORGE_ADDRESS)
            .typed(SovereignForgeProxy)
            .deploy_phase_one(
                min_validators,
                max_validators,
                min_stake,
                additional_stake_required,
            )
            .egld(payment);

        if let Some(error) = expected_result {
            transaction.returns(error).run();
        } else {
            transaction.run();
        }
    }

    fn deploy_phase_two(
        &mut self,
        expected_result: Option<ExpectError>,
        bls_keys: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>>,
    ) {
        let transaction = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(FORGE_ADDRESS)
            .typed(SovereignForgeProxy)
            .deploy_phase_two(bls_keys);

        if let Some(error) = expected_result {
            transaction.returns(error).run();
        } else {
            transaction.run();
        }
    }

    fn finish_setup(&mut self) {
        self.register_chain_factory(1, FACTORY_ADDRESS, None);
        self.register_chain_factory(2, FACTORY_ADDRESS, None);
        self.register_chain_factory(3, FACTORY_ADDRESS, None);
        self.register_token_handler(1, TOKEN_HANDLER_ADDRESS, None);
        self.register_token_handler(2, TOKEN_HANDLER_ADDRESS, None);
        self.register_token_handler(3, TOKEN_HANDLER_ADDRESS, None);
        self.complete_setup_phase(None);
    }
}

#[test]
fn test_deploy_contracts() {
    let mut state = SovereignForgeTestState::new();
    state.deploy_sovereign_forge();
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

    state.finish_setup();

    state
        .world
        .query()
        .to(FORGE_ADDRESS)
        .whitebox(sovereign_forge::contract_obj, |sc| {
            assert!(sc.is_setup_phase_complete());
        });
}

#[test]
fn deploy_phase_one_deploy_cost_too_low() {
    let mut state = SovereignForgeTestState::new();
    state.deploy_sovereign_forge();
    state.finish_setup();

    let deploy_cost = BigUint::from(1u32);

    state.deploy_phase_one(
        deploy_cost,
        1,
        2,
        BigUint::from(2u32),
        MultiValueEncoded::new(),
        Some(ExpectError(
            4,
            "The given deploy cost is not equal to the standard amount",
        )),
    );
}

#[test]
fn deploy_phase_one_chain_config_already_deployed() {
    let mut state = SovereignForgeTestState::new();
    state.deploy_sovereign_forge();
    state.deploy_chain_factory();
    state.deploy_chain_config_template();
    state.finish_setup();

    let deploy_cost = BigUint::from(100_000u32);

    state.deploy_phase_one(
        deploy_cost.clone(),
        1,
        2,
        BigUint::from(2u32),
        MultiValueEncoded::new(),
        None,
    );

    state.deploy_phase_one(
        deploy_cost,
        1,
        2,
        BigUint::from(2u32),
        MultiValueEncoded::new(),
        Some(ExpectError(
            4,
            "The Chain-Config contract is already deployed",
        )),
    );
}

#[test]
fn deploy_phase_one() {
    let mut state = SovereignForgeTestState::new();
    state.deploy_sovereign_forge();
    state.deploy_chain_factory();
    state.deploy_chain_config_template();
    state.finish_setup();

    let deploy_cost = BigUint::from(100_000u32);

    state.deploy_phase_one(
        deploy_cost,
        1,
        2,
        BigUint::from(2u32),
        MultiValueEncoded::new(),
        None,
    );

    state
        .world
        .query()
        .to(FORGE_ADDRESS)
        .whitebox(sovereign_forge::contract_obj, |sc| {
            let sovereign_mapper = sc.sovereigns_mapper(&OWNER_ADDRESS.to_managed_address());

            assert!(!sovereign_mapper.is_empty());

            let chain_id = sovereign_mapper.get();

            let is_chain_config_deployed = sc
                .sovereign_deployed_contracts(&chain_id)
                .iter()
                .any(|sc_info| sc_info.id == ScArray::ChainConfig);
            assert!(is_chain_config_deployed);
        })
}

#[test]
fn deploy_phase_two_without_first_phase() {
    let mut state = SovereignForgeTestState::new();
    state.deploy_sovereign_forge();
    state.deploy_chain_factory();
    state.finish_setup();

    let bls_keys = MultiValueEncoded::<StaticApi, ManagedBuffer<StaticApi>>::new();

    state.deploy_phase_two(
        Some(ExpectError(
            4,
            "The current caller has not deployed any Sovereign Chain",
        )),
        bls_keys,
    );
}

#[test]
fn deploy_phase_two() {
    let mut state = SovereignForgeTestState::new();
    state.deploy_sovereign_forge();
    state.deploy_chain_factory();
    state.deploy_chain_config_template();
    state.finish_setup();

    let deploy_cost = BigUint::from(100_000u32);

    state.deploy_phase_one(
        deploy_cost,
        1,
        2,
        BigUint::from(2u32),
        MultiValueEncoded::new(),
        None,
    );

    state.deploy_header_verifier_template();

    let mut bls_keys = MultiValueEncoded::new();
    bls_keys.push(ManagedBuffer::from("bls1"));
    bls_keys.push(ManagedBuffer::from("bls2"));

    state.deploy_phase_two(None, bls_keys);
}
