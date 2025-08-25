use common_test_setup::{
    base_setup::init::{AccountSetup, BaseSetup},
    constants::{
        CHAIN_FACTORY_SC_ADDRESS, ESDT_SAFE_ADDRESS, OWNER_ADDRESS, OWNER_BALANCE,
        SOVEREIGN_FORGE_SC_ADDRESS,
    },
};
use multiversx_sc::{
    imports::OptionalValue,
    types::{
        BigUint, ManagedAddress, ManagedVec, MultiValueEncoded, ReturnsHandledOrError,
        ReturnsResultUnmanaged, TestSCAddress, TestTokenIdentifier,
    },
};
use multiversx_sc_scenario::{api::StaticApi, ScenarioTxRun};
use proxies::sovereign_forge_proxy::SovereignForgeProxy;
use sovereign_forge::common::storage::ChainId;
use structs::{
    configs::{EsdtSafeConfig, SovereignConfig},
    fee::FeeStruct,
    forge::ScArray,
};

pub struct SovereignForgeTestState {
    pub common_setup: BaseSetup,
}

impl SovereignForgeTestState {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let owner_setup = AccountSetup {
            address: OWNER_ADDRESS.to_address(),
            code_path: None,
            esdt_balances: None,
            egld_balance: Some(BigUint::from(OWNER_BALANCE)),
        };

        let account_setups = vec![owner_setup];

        let common_setup = BaseSetup::new(account_setups);

        Self { common_setup }
    }

    pub fn finish_setup(&mut self) {
        self.register_chain_factory(1, CHAIN_FACTORY_SC_ADDRESS, None);
        self.register_chain_factory(2, CHAIN_FACTORY_SC_ADDRESS, None);
        self.register_chain_factory(3, CHAIN_FACTORY_SC_ADDRESS, None);
    }

    pub fn deploy_template_scs(&mut self, templates: Option<Vec<ScArray>>) {
        for sc in templates.unwrap_or_default().into_iter() {
            match sc {
                ScArray::ChainConfig => {
                    self.common_setup
                        .deploy_chain_config(OptionalValue::None, None);
                }
                ScArray::ESDTSafe => {
                    self.common_setup.deploy_mvx_esdt_safe(OptionalValue::None);
                }
                ScArray::FeeMarket => {
                    self.common_setup.deploy_fee_market(None, ESDT_SAFE_ADDRESS);
                }
                ScArray::HeaderVerifier => {
                    self.common_setup.deploy_header_verifier(vec![]);
                }
                ScArray::ChainFactory => {
                    self.common_setup.deploy_chain_factory();
                }
                _ => {}
            }
        }
    }

    pub fn register_chain_factory(
        &mut self,
        shard_id: u32,
        chain_factory_address: TestSCAddress,
        error_message: Option<&str>,
    ) {
        let response = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(SOVEREIGN_FORGE_SC_ADDRESS)
            .typed(SovereignForgeProxy)
            .register_chain_factory(shard_id, chain_factory_address)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(response, error_message);
    }

    pub fn update_sovereign_config(
        &mut self,
        new_sovereign_config: SovereignConfig<StaticApi>,
        expected_error_message: Option<&str>,
    ) {
        let response = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(SOVEREIGN_FORGE_SC_ADDRESS)
            .typed(SovereignForgeProxy)
            .update_sovereign_config(new_sovereign_config)
            .returns(ReturnsHandledOrError::new())
            .run();

        if let Err(error) = response {
            assert_eq!(expected_error_message, Some(error.message.as_str()))
        }
    }

    pub fn update_esdt_safe_config(
        &mut self,
        new_esdt_safe_config: EsdtSafeConfig<StaticApi>,
        expected_error_message: Option<&str>,
    ) {
        let response = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(SOVEREIGN_FORGE_SC_ADDRESS)
            .typed(SovereignForgeProxy)
            .update_esdt_safe_config(new_esdt_safe_config)
            .returns(ReturnsHandledOrError::new())
            .run();

        if let Err(error) = response {
            assert_eq!(expected_error_message, Some(error.message.as_str()))
        }
    }

    pub fn set_fee(&mut self, new_fee: FeeStruct<StaticApi>, expected_error_message: Option<&str>) {
        let response = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(SOVEREIGN_FORGE_SC_ADDRESS)
            .typed(SovereignForgeProxy)
            .set_fee(new_fee)
            .returns(ReturnsHandledOrError::new())
            .run();

        if let Err(error) = response {
            assert_eq!(expected_error_message, Some(error.message.as_str()))
        }
    }

    pub fn remove_fee(
        &mut self,
        token_id: TestTokenIdentifier,
        expected_error_message: Option<&str>,
    ) {
        let response = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(SOVEREIGN_FORGE_SC_ADDRESS)
            .typed(SovereignForgeProxy)
            .remove_fee(token_id)
            .returns(ReturnsHandledOrError::new())
            .run();

        if let Err(error) = response {
            assert_eq!(expected_error_message, Some(error.message.as_str()))
        }
    }

    pub fn complete_setup_phase(&mut self, error_message: Option<&str>) {
        let response = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(SOVEREIGN_FORGE_SC_ADDRESS)
            .typed(SovereignForgeProxy)
            .complete_setup_phase()
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(response, error_message);
    }

    pub fn add_users_to_whitelist(
        &mut self,
        users: Vec<ManagedAddress<StaticApi>>,
        error_message: Option<&str>,
    ) {
        let response = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(SOVEREIGN_FORGE_SC_ADDRESS)
            .typed(SovereignForgeProxy)
            .add_users_to_whitelist(MultiValueEncoded::from(ManagedVec::from_iter(users)))
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(response, error_message);
    }

    pub fn remove_users_from_whitelist(
        &mut self,
        users: Vec<ManagedAddress<StaticApi>>,
        error_message: Option<&str>,
    ) {
        let response = self
            .common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(SOVEREIGN_FORGE_SC_ADDRESS)
            .typed(SovereignForgeProxy)
            .remove_users_from_whitelist(MultiValueEncoded::from(ManagedVec::from_iter(users)))
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(response, error_message);
    }

    pub fn get_smart_contract_address_from_sovereign_forge(
        &mut self,
        chain_id: ChainId<StaticApi>,
        sc_id: ScArray,
    ) -> ManagedAddress<StaticApi> {
        self.common_setup
            .world
            .query()
            .to(SOVEREIGN_FORGE_SC_ADDRESS)
            .typed(SovereignForgeProxy)
            .sovereign_deployed_contracts(chain_id)
            .returns(ReturnsResultUnmanaged)
            .run()
            .iter()
            .find(|sc| sc.id == sc_id)
            .unwrap()
            .address
            .clone()
    }

    pub fn check_setup_phase_completed(
        &mut self,
        chain_id: ChainId<StaticApi>,
        expected_result: bool,
    ) {
        let response = self
            .common_setup
            .world
            .query()
            .to(SOVEREIGN_FORGE_SC_ADDRESS)
            .typed(SovereignForgeProxy)
            .sovereign_setup_phase(chain_id)
            .returns(ReturnsResultUnmanaged)
            .run();

        assert_eq!(response, expected_result);
    }
}
