#[multiversx_sc::module]
pub trait CompletePhasesModule {
    #[endpoint(completeChainConfigSetup)]
    fn complete_chain_config_setup(&self) {}

    #[endpoint(completeHeaderVerifierSetup)]
    fn complete_header_verifier_setup(&self) {}

    #[endpoint(completeFeeMarketSetup)]
    fn complete_fee_market_setup(&self) {}

    #[endpoint(completeEsdtSafeSetup)]
    fn complete_esdt_safe_setup(&self) {}
}
