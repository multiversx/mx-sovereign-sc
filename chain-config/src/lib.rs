#![no_std]

use multiversx_sc::imports::*;
use structs::configs::SovereignConfig;

multiversx_sc::imports!();

pub mod config_utils;
pub mod configs;
pub mod storage;
pub mod validator;

#[multiversx_sc::contract]
pub trait ChainConfigContract:
    validator::ValidatorModule
    + storage::ChainConfigStorageModule
    + config_utils::ChainConfigUtilsModule
    + configs::ConfigsModule
    + setup_phase::SetupPhaseModule
    + common_utils::CommonUtilsModule
    + custom_events::CustomEventsModule
{
    #[init]
    fn init(&self, opt_config: OptionalValue<SovereignConfig<Self::Api>>) {
        let new_config = match opt_config {
            OptionalValue::Some(cfg) => {
                if let Some(error_message) = self.is_new_config_valid(&cfg) {
                    sc_panic!(error_message);
                }
                cfg
            }
            OptionalValue::None => SovereignConfig::default_config(),
        };

        self.sovereign_config().set(new_config.clone());
    }

    #[upgrade]
    fn upgrade(&self) {}

    #[only_owner]
    #[endpoint(completeSetupPhase)]
    fn complete_setup_phase(&self) {
        if self.is_setup_phase_complete() {
            return;
        }
        self.require_validator_set_valid(self.bls_keys_map().len());

        self.complete_genesis_event();
        self.setup_phase_complete().set(true);
    }
}
