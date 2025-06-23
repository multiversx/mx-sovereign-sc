use multiversx_sc_scenario::{ReturnsHandledOrError, ReturnsLogs, ScenarioTxRun};
use proxies::{
    chain_config_proxy::ChainConfigContractProxy, fee_market_proxy::FeeMarketProxy,
    header_verifier_proxy::HeaderverifierProxy, sovereign_forge_proxy::SovereignForgeProxy,
};

use crate::{
    base_setup::init::BaseSetup,
    constants::{
        CHAIN_CONFIG_ADDRESS, FEE_MARKET_ADDRESS, HEADER_VERIFIER_ADDRESS, OWNER_ADDRESS,
        SOVEREIGN_FORGE_SC_ADDRESS,
    },
};

impl BaseSetup {
    pub fn complete_header_verifier_setup_phase(&mut self, expected_error_message: Option<&str>) {
        let response = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(HEADER_VERIFIER_ADDRESS)
            .typed(HeaderverifierProxy)
            .complete_setup_phase()
            .returns(ReturnsHandledOrError::new())
            .run();

        self.assert_expected_error_message(response, expected_error_message);
    }

    pub fn complete_fee_market_setup_phase(&mut self, expected_error_message: Option<&str>) {
        let response = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(FEE_MARKET_ADDRESS)
            .typed(FeeMarketProxy)
            .complete_setup_phase()
            .returns(ReturnsHandledOrError::new())
            .run();

        self.change_ownership_to_header_verifier(FEE_MARKET_ADDRESS);

        self.assert_expected_error_message(response, expected_error_message);
    }

    pub fn complete_sovereign_forge_setup_phase(&mut self, expected_error_message: Option<&str>) {
        let response = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(SOVEREIGN_FORGE_SC_ADDRESS)
            .typed(SovereignForgeProxy)
            .complete_setup_phase()
            .returns(ReturnsHandledOrError::new())
            .run();

        self.assert_expected_error_message(response, expected_error_message);
    }

    pub fn complete_chain_config_setup_phase(&mut self, expect_error: Option<&str>) {
        let transaction = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(CHAIN_CONFIG_ADDRESS)
            .typed(ChainConfigContractProxy)
            .complete_setup_phase()
            .returns(ReturnsHandledOrError::new())
            .run();

        self.assert_expected_error_message(transaction, expect_error);
    }

    pub fn complete_chain_config_genesis_phase(
        &mut self,
        expect_error: Option<&str>,
        expected_log: Option<&str>,
    ) {
        let (transaction, logs) = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(CHAIN_CONFIG_ADDRESS)
            .typed(ChainConfigContractProxy)
            .complete_genesis()
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run();

        self.assert_expected_error_message(transaction, expect_error);

        self.assert_expected_log(logs, expected_log);
    }
}
