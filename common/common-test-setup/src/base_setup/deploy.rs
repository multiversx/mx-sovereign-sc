use multiversx_sc_scenario::{
    api::StaticApi,
    imports::{
        BigUint, ManagedBuffer, MultiValueEncoded, OptionalValue, TestSCAddress, TokenIdentifier,
    },
    ReturnsHandledOrError, ScenarioTxRun,
};
use proxies::{
    chain_config_proxy::ChainConfigContractProxy, chain_factory_proxy::ChainFactoryContractProxy,
    enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy, fee_market_proxy::FeeMarketProxy,
    header_verifier_proxy::HeaderverifierProxy, mvx_esdt_safe_proxy::MvxEsdtSafeProxy,
    sov_esdt_safe_proxy::SovEsdtSafeProxy, sovereign_forge_proxy::SovereignForgeProxy,
    testing_sc_proxy::TestingScProxy, token_handler_proxy::TokenHandlerProxy,
};
use structs::{
    configs::{EsdtSafeConfig, SovereignConfig},
    fee::FeeStruct,
    forge::ScArray,
};

use crate::{
    base_setup::init::BaseSetup,
    constants::{
        CHAIN_CONFIG_ADDRESS, CHAIN_CONFIG_CODE_PATH, CHAIN_FACTORY_CODE_PATH,
        CHAIN_FACTORY_SC_ADDRESS, DEPLOY_COST, ENSHRINE_ESDT_SAFE_CODE_PATH, ENSHRINE_SC_ADDRESS,
        ESDT_SAFE_ADDRESS, FEE_MARKET_ADDRESS, FEE_MARKET_CODE_PATH, HEADER_VERIFIER_ADDRESS,
        HEADER_VERIFIER_CODE_PATH, MVX_ESDT_SAFE_CODE_PATH, OWNER_ADDRESS,
        SOVEREIGN_FORGE_CODE_PATH, SOVEREIGN_FORGE_SC_ADDRESS, SOV_ESDT_SAFE_CODE_PATH,
        TESTING_SC_ADDRESS, TESTING_SC_CODE_PATH, TOKEN_HANDLER_CODE_PATH,
        TOKEN_HANDLER_SC_ADDRESS,
    },
};

impl BaseSetup {
    pub fn deploy_mvx_esdt_safe(
        &mut self,
        opt_config: OptionalValue<EsdtSafeConfig<StaticApi>>,
    ) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(MvxEsdtSafeProxy)
            .init(opt_config)
            .code(MVX_ESDT_SAFE_CODE_PATH)
            .new_address(ESDT_SAFE_ADDRESS)
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(MvxEsdtSafeProxy)
            .unpause_endpoint()
            .run();

        self
    }

    pub fn deploy_fee_market(
        &mut self,
        fee: Option<FeeStruct<StaticApi>>,
        esdt_safe_address: TestSCAddress,
    ) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(FeeMarketProxy)
            .init(esdt_safe_address, fee)
            .code(FEE_MARKET_CODE_PATH)
            .new_address(FEE_MARKET_ADDRESS)
            .run();

        self
    }

    pub fn deploy_testing_sc(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(TestingScProxy)
            .init()
            .code(TESTING_SC_CODE_PATH)
            .new_address(TESTING_SC_ADDRESS)
            .run();

        self
    }

    pub fn deploy_header_verifier(&mut self, sovereign_contracts: Vec<ScArray>) -> &mut Self {
        let contracts_array = self.get_contract_info_struct_for_sc_type(sovereign_contracts);

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(HeaderverifierProxy)
            .init(MultiValueEncoded::from_iter(contracts_array))
            .code(HEADER_VERIFIER_CODE_PATH)
            .new_address(HEADER_VERIFIER_ADDRESS)
            .run();

        self
    }

    pub fn deploy_chain_config(
        &mut self,
        opt_config: OptionalValue<SovereignConfig<StaticApi>>,
        expected_error_message: Option<&str>,
    ) -> &mut Self {
        let response = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(ChainConfigContractProxy)
            .init(opt_config)
            .code(CHAIN_CONFIG_CODE_PATH)
            .new_address(CHAIN_CONFIG_ADDRESS)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.assert_expected_error_message(response, expected_error_message);

        self
    }

    pub fn deploy_chain_factory(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(ChainFactoryContractProxy)
            .init(
                SOVEREIGN_FORGE_SC_ADDRESS,
                CHAIN_CONFIG_ADDRESS,
                HEADER_VERIFIER_ADDRESS,
                ESDT_SAFE_ADDRESS,
                FEE_MARKET_ADDRESS,
            )
            .code(CHAIN_FACTORY_CODE_PATH)
            .new_address(CHAIN_FACTORY_SC_ADDRESS)
            .run();

        self
    }

    pub fn deploy_sovereign_forge(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(SovereignForgeProxy)
            .init(DEPLOY_COST)
            .code(SOVEREIGN_FORGE_CODE_PATH)
            .new_address(SOVEREIGN_FORGE_SC_ADDRESS)
            .run();

        self
    }

    pub fn deploy_enshrine_esdt_contract(
        &mut self,
        is_sovereign_chain: bool,
        wegld_identifier: Option<TokenIdentifier<StaticApi>>,
        sovereign_token_prefix: Option<ManagedBuffer<StaticApi>>,
        opt_config: Option<EsdtSafeConfig<StaticApi>>,
    ) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(EnshrineEsdtSafeProxy)
            .init(
                is_sovereign_chain,
                TOKEN_HANDLER_SC_ADDRESS,
                wegld_identifier,
                sovereign_token_prefix,
                opt_config,
            )
            .code(ENSHRINE_ESDT_SAFE_CODE_PATH)
            .new_address(ENSHRINE_SC_ADDRESS)
            .run();

        self
    }

    pub fn deploy_token_handler(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(TokenHandlerProxy)
            .init(CHAIN_FACTORY_SC_ADDRESS)
            .code(TOKEN_HANDLER_CODE_PATH)
            .new_address(TOKEN_HANDLER_SC_ADDRESS)
            .run();

        self
    }

    pub fn deploy_sov_esdt_safe(
        &mut self,
        fee_market_address: TestSCAddress,
        opt_config: OptionalValue<EsdtSafeConfig<StaticApi>>,
    ) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(SovEsdtSafeProxy)
            .init(fee_market_address, opt_config)
            .code(SOV_ESDT_SAFE_CODE_PATH)
            .new_address(ESDT_SAFE_ADDRESS)
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(SovEsdtSafeProxy)
            .unpause_endpoint()
            .run();

        self
    }

    pub fn deploy_phase_one(
        &mut self,
        payment: &BigUint<StaticApi>,
        opt_preferred_chain: Option<ManagedBuffer<StaticApi>>,
        opt_config: OptionalValue<SovereignConfig<StaticApi>>,
        error_message: Option<&str>,
    ) {
        let response = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(SOVEREIGN_FORGE_SC_ADDRESS)
            .typed(SovereignForgeProxy)
            .deploy_phase_one(opt_preferred_chain, opt_config)
            .egld(payment)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.assert_expected_error_message(response, error_message);
    }

    pub fn deploy_phase_two(
        &mut self,
        opt_config: OptionalValue<EsdtSafeConfig<StaticApi>>,
        error_message: Option<&str>,
    ) {
        let response = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(SOVEREIGN_FORGE_SC_ADDRESS)
            .typed(SovereignForgeProxy)
            .deploy_phase_two(opt_config)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.assert_expected_error_message(response, error_message);
    }

    pub fn deploy_phase_three(
        &mut self,
        fee: Option<FeeStruct<StaticApi>>,
        error_message: Option<&str>,
    ) {
        let response = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(SOVEREIGN_FORGE_SC_ADDRESS)
            .typed(SovereignForgeProxy)
            .deploy_phase_three(fee)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.assert_expected_error_message(response, error_message);
    }

    pub fn deploy_phase_four(&mut self, error_message: Option<&str>) {
        let response = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(SOVEREIGN_FORGE_SC_ADDRESS)
            .typed(SovereignForgeProxy)
            .deploy_phase_four()
            .returns(ReturnsHandledOrError::new())
            .run();

        self.assert_expected_error_message(response, error_message);
    }
}
