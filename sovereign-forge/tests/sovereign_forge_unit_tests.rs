use common_test_setup::constants::CHAIN_CONFIG_ADDRESS;
use multiversx_sc::{
    imports::OptionalValue,
    types::{BigUint, ManagedBuffer, TestAddress, TestSCAddress},
};
use multiversx_sc_scenario::{
    api::StaticApi, imports::MxscPath, ReturnsHandledOrError, ScenarioTxRun, ScenarioTxWhitebox,
    ScenarioWorld,
};
use proxies::{
    chain_config_proxy::ChainConfigContractProxy,
    chain_factory_proxy::ChainFactoryContractProxy,
    fee_market_proxy::{FeeMarketProxy, FeeStruct},
    header_verifier_proxy::HeaderverifierProxy,
    mvx_esdt_safe_proxy::MvxEsdtSafeProxy,
    sovereign_forge_proxy::SovereignForgeProxy,
};
use setup_phase::SetupPhaseModule;
use sovereign_forge::common::{
    storage::StorageModule,
    utils::{ScArray, UtilsModule},
};
use structs::configs::{EsdtSafeConfig, SovereignConfig};

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

const ESDT_SAFE_ADDRESS: TestSCAddress = TestSCAddress::new("esdt-safe");
const ESDT_SAFE_CODE_PATH: MxscPath = MxscPath::new("../esdt-safe/output/esdt-safe.mxsc.json");

const FEE_MARKET_ADDRESS: TestSCAddress = TestSCAddress::new("fee-market");
const FEE_MARKET_CODE_PATH: MxscPath = MxscPath::new("../fee-market/output/fee-market.mxsc.json");

const TOKEN_HANDLER_ADDRESS: TestSCAddress = TestSCAddress::new("token-handler");

const BALANCE: u128 = 100_000_000_000_000_000;
const DEPLOY_COST: u64 = 100_000;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(FORGE_CODE_PATH, sovereign_forge::ContractBuilder);
    blockchain.register_contract(FACTORY_CODE_PATH, chain_factory::ContractBuilder);
    blockchain.register_contract(CONFIG_CODE_PATH, chain_config::ContractBuilder);
    blockchain.register_contract(HEADER_VERIFIER_CODE_PATH, header_verifier::ContractBuilder);
    blockchain.register_contract(ESDT_SAFE_CODE_PATH, mvx_esdt_safe::ContractBuilder);
    blockchain.register_contract(FEE_MARKET_CODE_PATH, fee_market::ContractBuilder);

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

    fn finish_setup(&mut self) {
        self.register_chain_factory(1, FACTORY_ADDRESS, None);
        self.register_chain_factory(2, FACTORY_ADDRESS, None);
        self.register_chain_factory(3, FACTORY_ADDRESS, None);
        self.register_token_handler(1, TOKEN_HANDLER_ADDRESS, None);
        self.register_token_handler(2, TOKEN_HANDLER_ADDRESS, None);
        self.register_token_handler(3, TOKEN_HANDLER_ADDRESS, None);
        self.complete_setup_phase(None);
    }

    fn deploy_chain_factory(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(FORGE_ADDRESS)
            .typed(ChainFactoryContractProxy)
            .init(
                FORGE_ADDRESS,
                CONFIG_ADDRESS,
                HEADER_VERIFIER_ADDRESS,
                ESDT_SAFE_ADDRESS,
                FEE_MARKET_ADDRESS,
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
            .init(SovereignConfig::default_config(), OWNER_ADDRESS)
            .code(CONFIG_CODE_PATH)
            .new_address(CONFIG_ADDRESS)
            .run();

        self
    }

    fn deploy_header_verifier_template(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(HeaderverifierProxy)
            .init(CHAIN_CONFIG_ADDRESS)
            .code(HEADER_VERIFIER_CODE_PATH)
            .new_address(HEADER_VERIFIER_ADDRESS)
            .run();

        self
    }

    // TODO: MVX-ESDT-SAFE
    fn deploy_mvx_esdt_safe_template(
        &mut self,
        header_verifier_address: &TestSCAddress,
        opt_config: OptionalValue<EsdtSafeConfig<StaticApi>>,
    ) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(MvxEsdtSafeProxy)
            .init(header_verifier_address.to_managed_address(), opt_config)
            .code(ESDT_SAFE_CODE_PATH)
            .new_address(ESDT_SAFE_ADDRESS)
            .run();

        self
    }

    fn deploy_fee_market_template(&mut self) -> &mut Self {
        let fee: Option<FeeStruct<StaticApi>> = None;

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(FeeMarketProxy)
            .init(ESDT_SAFE_ADDRESS, fee)
            .code(FEE_MARKET_CODE_PATH)
            .new_address(FEE_MARKET_ADDRESS)
            .run();

        self
    }

    fn register_token_handler(
        &mut self,
        shard_id: u32,
        token_handler_address: TestSCAddress,
        error_message: Option<&str>,
    ) {
        let response = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(FORGE_ADDRESS)
            .typed(SovereignForgeProxy)
            .register_token_handler(shard_id, token_handler_address)
            .returns(ReturnsHandledOrError::new())
            .run();

        if let Err(error) = response {
            assert_eq!(error_message, Some(error.message.as_str()))
        }
    }

    fn register_chain_factory(
        &mut self,
        shard_id: u32,
        chain_factory_address: TestSCAddress,
        error_message: Option<&str>,
    ) {
        let response = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(FORGE_ADDRESS)
            .typed(SovereignForgeProxy)
            .register_chain_factory(shard_id, chain_factory_address)
            .returns(ReturnsHandledOrError::new())
            .run();

        if let Err(error) = response {
            assert_eq!(error_message, Some(error.message.as_str()))
        }
    }

    fn complete_setup_phase(&mut self, error_message: Option<&str>) {
        let response = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(FORGE_ADDRESS)
            .typed(SovereignForgeProxy)
            .complete_setup_phase()
            .returns(ReturnsHandledOrError::new())
            .run();

        if let Err(error) = response {
            assert_eq!(error_message, Some(error.message.as_str()))
        }
    }

    fn deploy_phase_one(
        &mut self,
        payment: &BigUint<StaticApi>,
        opt_preferred_chain: Option<ManagedBuffer<StaticApi>>,
        config: &SovereignConfig<StaticApi>,
        error_message: Option<&str>,
    ) {
        let response = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(FORGE_ADDRESS)
            .typed(SovereignForgeProxy)
            .deploy_phase_one(opt_preferred_chain, config)
            .egld(payment)
            .returns(ReturnsHandledOrError::new())
            .run();

        if let Err(error) = response {
            assert_eq!(error_message, Some(error.message.as_str()))
        }
    }

    fn deploy_phase_two(&mut self, error_message: Option<&str>) {
        let response = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(FORGE_ADDRESS)
            .typed(SovereignForgeProxy)
            .deploy_phase_two()
            .returns(ReturnsHandledOrError::new())
            .run();

        if let Err(error) = response {
            assert_eq!(error_message, Some(error.message.as_str()))
        }
    }

    fn deploy_phase_three(
        &mut self,
        opt_config: OptionalValue<EsdtSafeConfig<StaticApi>>,
        error_message: Option<&str>,
    ) {
        let response = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(FORGE_ADDRESS)
            .typed(SovereignForgeProxy)
            .deploy_phase_three(opt_config)
            .returns(ReturnsHandledOrError::new())
            .run();

        if let Err(error) = response {
            assert_eq!(error_message, Some(error.message.as_str()))
        }
    }

    fn deploy_phase_four(
        &mut self,
        fee: Option<FeeStruct<StaticApi>>,
        error_message: Option<&str>,
    ) {
        let response = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(FORGE_ADDRESS)
            .typed(SovereignForgeProxy)
            .deploy_phase_four(fee)
            .returns(ReturnsHandledOrError::new())
            .run();

        if let Err(error) = response {
            assert_eq!(error_message, Some(error.message.as_str()))
        }
    }
}

#[test]
fn deploy_contracts() {
    let mut state = SovereignForgeTestState::new();
    state.deploy_sovereign_forge();
    state.deploy_chain_factory();
}

#[test]
fn register_token_handler() {
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
fn register_chain_factory() {
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

    state.complete_setup_phase(Some(
        "There is no Chain-Factory contract assigned for shard 1",
    ));
}

#[test]
fn complete_setup_phase_no_token_handler_registered() {
    let mut state = SovereignForgeTestState::new();
    state.deploy_sovereign_forge();
    state.register_chain_factory(1, FACTORY_ADDRESS, None);

    state.complete_setup_phase(Some(
        "There is no Token-Handler contract assigned for shard 1",
    ));
}

#[test]
fn complete_setup_phase() {
    let mut state = SovereignForgeTestState::new();
    state.deploy_sovereign_forge();
    state.deploy_chain_factory();

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
    state.deploy_chain_factory();
    state.finish_setup();

    let deploy_cost = BigUint::from(1u32);

    state.deploy_phase_one(
        &deploy_cost,
        None,
        &SovereignConfig::default_config(),
        Some("The given deploy cost is not equal to the standard amount"),
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
    let config = SovereignConfig::default_config();

    state.deploy_phase_one(&deploy_cost, None, &config, None);
    state.deploy_phase_one(
        &deploy_cost,
        None,
        &config,
        Some("The Chain-Config contract is already deployed"),
    );
}

#[test]
fn deploy_phase_one_no_preferred_chain_id() {
    let mut state = SovereignForgeTestState::new();
    state.deploy_sovereign_forge();
    state.deploy_chain_factory();
    state.deploy_chain_config_template();
    state.finish_setup();

    let deploy_cost = BigUint::from(100_000u32);

    state.deploy_phase_one(&deploy_cost, None, &SovereignConfig::default_config(), None);

    state
        .world
        .query()
        .to(FORGE_ADDRESS)
        .whitebox(sovereign_forge::contract_obj, |sc| {
            assert!(!sc
                .sovereigns_mapper(&OWNER_ADDRESS.to_managed_address())
                .is_empty());

            let is_chain_config_deployed =
                sc.is_contract_deployed(&OWNER_ADDRESS.to_managed_address(), ScArray::ChainConfig);
            assert!(is_chain_config_deployed);
        })
}

#[test]
fn deploy_phase_one_preferred_chain_id() {
    let mut state = SovereignForgeTestState::new();
    state.deploy_sovereign_forge();
    state.deploy_chain_factory();
    state.deploy_chain_config_template();
    state.finish_setup();

    let deploy_cost = BigUint::from(100_000u32);

    state.deploy_phase_one(
        &deploy_cost,
        Some(ManagedBuffer::from("SVCH")),
        &SovereignConfig::default_config(),
        None,
    );

    state
        .world
        .query()
        .to(FORGE_ADDRESS)
        .whitebox(sovereign_forge::contract_obj, |sc| {
            assert!(!sc
                .sovereigns_mapper(&OWNER_ADDRESS.to_managed_address())
                .is_empty());

            assert!(sc.chain_ids().contains(&ManagedBuffer::from("SVCH")));

            let is_chain_config_deployed =
                sc.is_contract_deployed(&OWNER_ADDRESS.to_managed_address(), ScArray::ChainConfig);
            assert!(is_chain_config_deployed);
        })
}

#[test]
fn deploy_phase_one_with_chain_id_used() {
    let mut state = SovereignForgeTestState::new();
    state.deploy_sovereign_forge();
    state.deploy_chain_factory();
    state.deploy_chain_config_template();
    state.finish_setup();

    let deploy_cost = BigUint::from(100_000u32);

    state.deploy_phase_one(
        &deploy_cost,
        Some(ManagedBuffer::from("SVCH")),
        &SovereignConfig::default_config(),
        None,
    );

    state.deploy_phase_one(
        &deploy_cost,
        Some(ManagedBuffer::from("SVCH")),
        &SovereignConfig::default_config(),
        Some("This chain ID is already used"),
    );
}
#[test]
fn deploy_phase_two_without_first_phase() {
    let mut state = SovereignForgeTestState::new();
    state.deploy_sovereign_forge();
    state.deploy_chain_factory();
    state.finish_setup();

    state.deploy_phase_two(Some(
        "The current caller has not deployed any Sovereign Chain",
    ));
}

#[test]
fn deploy_phase_two() {
    let mut state = SovereignForgeTestState::new();
    state.deploy_sovereign_forge();
    state.deploy_chain_factory();
    state.deploy_chain_config_template();
    state.finish_setup();

    let deploy_cost = BigUint::from(100_000u32);

    state.deploy_phase_one(&deploy_cost, None, &SovereignConfig::default_config(), None);
    state.deploy_header_verifier_template();

    state.deploy_phase_two(None);

    state
        .world
        .query()
        .to(FORGE_ADDRESS)
        .whitebox(sovereign_forge::contract_obj, |sc| {
            let is_header_verifier_deployed = sc
                .is_contract_deployed(&OWNER_ADDRESS.to_managed_address(), ScArray::HeaderVerifier);

            assert!(is_header_verifier_deployed);
        })
}

#[test]
fn deploy_phase_two_header_already_deployed() {
    let mut state = SovereignForgeTestState::new();
    state.deploy_sovereign_forge();
    state.deploy_chain_factory();
    state.deploy_chain_config_template();
    state.finish_setup();

    let deploy_cost = BigUint::from(100_000u32);

    state.deploy_phase_one(&deploy_cost, None, &SovereignConfig::default_config(), None);
    state.deploy_header_verifier_template();

    state.deploy_phase_two(None);
    state.deploy_phase_two(Some("The Header-Verifier contract is already deployed"));
}

#[test]
fn deploy_phase_three() {
    let mut state = SovereignForgeTestState::new();
    state.deploy_sovereign_forge();
    state.deploy_chain_factory();
    state.deploy_chain_config_template();
    state.finish_setup();

    let deploy_cost = BigUint::from(100_000u32);

    state.deploy_phase_one(&deploy_cost, None, &SovereignConfig::default_config(), None);
    state.deploy_header_verifier_template();
    state.deploy_mvx_esdt_safe_template(&HEADER_VERIFIER_ADDRESS, OptionalValue::None);

    state.deploy_phase_two(None);
    state.deploy_phase_three(OptionalValue::None, None);

    state
        .world
        .query()
        .to(FORGE_ADDRESS)
        .whitebox(sovereign_forge::contract_obj, |sc| {
            let is_esdt_safe_deployed =
                sc.is_contract_deployed(&OWNER_ADDRESS.to_managed_address(), ScArray::ESDTSafe);

            assert!(is_esdt_safe_deployed);
        })
}

#[test]
fn deploy_phase_three_without_phase_one() {
    let mut state = SovereignForgeTestState::new();
    state.deploy_sovereign_forge();
    state.deploy_chain_factory();
    state.deploy_chain_config_template();
    state.finish_setup();

    state.deploy_phase_three(
        OptionalValue::None,
        Some("The Header-Verifier SC is not deployed, you skipped the second phase"),
    );
}

#[test]
fn deploy_phase_three_without_phase_two() {
    let mut state = SovereignForgeTestState::new();
    state.deploy_sovereign_forge();
    state.deploy_chain_factory();
    state.deploy_chain_config_template();
    state.finish_setup();

    let deploy_cost = BigUint::from(100_000u32);
    state.deploy_phase_one(&deploy_cost, None, &SovereignConfig::default_config(), None);

    state.deploy_header_verifier_template();
    state.deploy_mvx_esdt_safe_template(&HEADER_VERIFIER_ADDRESS, OptionalValue::None);

    state.deploy_phase_three(
        OptionalValue::None,
        Some("The Header-Verifier SC is not deployed, you skipped the second phase"),
    );
}

#[test]
fn deploy_phase_three_already_deployed() {
    let mut state = SovereignForgeTestState::new();
    state.deploy_sovereign_forge();
    state.deploy_chain_factory();
    state.deploy_chain_config_template();
    state.finish_setup();

    let deploy_cost = BigUint::from(100_000u32);
    state.deploy_phase_one(&deploy_cost, None, &SovereignConfig::default_config(), None);

    state.deploy_header_verifier_template();
    state.deploy_mvx_esdt_safe_template(&HEADER_VERIFIER_ADDRESS, OptionalValue::None);

    state.deploy_phase_two(None);
    state.deploy_phase_three(OptionalValue::None, None);
    state.deploy_phase_three(
        OptionalValue::None,
        Some("The ESDT-Safe SC is already deployed"),
    );
}

#[test]
fn deploy_phase_four() {
    let mut state = SovereignForgeTestState::new();
    state.deploy_sovereign_forge();
    state.deploy_chain_factory();
    state.deploy_chain_config_template();
    state.deploy_fee_market_template();
    state.finish_setup();

    let deploy_cost = BigUint::from(100_000u32);
    state.deploy_phase_one(&deploy_cost, None, &SovereignConfig::default_config(), None);

    state.deploy_header_verifier_template();
    state.deploy_mvx_esdt_safe_template(&HEADER_VERIFIER_ADDRESS, OptionalValue::None);

    state.deploy_phase_two(None);
    state.deploy_phase_three(OptionalValue::None, None);
    state.deploy_phase_four(None, None);

    state
        .world
        .query()
        .to(FORGE_ADDRESS)
        .whitebox(sovereign_forge::contract_obj, |sc| {
            let is_fee_market_deployed =
                sc.is_contract_deployed(&OWNER_ADDRESS.to_managed_address(), ScArray::FeeMarket);

            assert!(is_fee_market_deployed);
        })
}

#[test]
fn deploy_phase_four_without_previous_phase() {
    let mut state = SovereignForgeTestState::new();
    state.deploy_sovereign_forge();
    state.deploy_chain_factory();
    state.deploy_chain_config_template();
    state.deploy_fee_market_template();
    state.finish_setup();

    let deploy_cost = BigUint::from(100_000u32);
    state.deploy_phase_one(&deploy_cost, None, &SovereignConfig::default_config(), None);

    state.deploy_header_verifier_template();
    state.deploy_mvx_esdt_safe_template(&HEADER_VERIFIER_ADDRESS, OptionalValue::None);

    state.deploy_phase_two(None);
    state.deploy_phase_four(
        None,
        Some("The ESDT-Safe SC is not deployed, you skipped the third phase"),
    );
}

#[test]
fn deploy_phase_four_fee_market_already_deployed() {
    let mut state = SovereignForgeTestState::new();
    state.deploy_sovereign_forge();
    state.deploy_chain_factory();
    state.deploy_chain_config_template();
    state.deploy_fee_market_template();
    state.finish_setup();

    let deploy_cost = BigUint::from(100_000u32);
    state.deploy_phase_one(&deploy_cost, None, &SovereignConfig::default_config(), None);

    state.deploy_header_verifier_template();
    state.deploy_mvx_esdt_safe_template(&HEADER_VERIFIER_ADDRESS, OptionalValue::None);

    state.deploy_phase_two(None);
    state.deploy_phase_three(OptionalValue::None, None);
    state.deploy_phase_four(None, None);
    state.deploy_phase_four(None, Some("The Fee-Market SC is already deployed"));
}
