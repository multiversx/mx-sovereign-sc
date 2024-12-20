#![no_std]

use multiversx_sc_modules::only_admin;
use transaction::SovereignConfig;

multiversx_sc::imports!();

pub mod validator_rules;

#[multiversx_sc::contract]
pub trait ChainConfigContract:
    validator_rules::ValidatorRulesModule + only_admin::OnlyAdminModule + setup_phase::SetupPhaseModule
{
    #[init]
    fn init(&self, config: SovereignConfig<Self::Api>, admin: ManagedAddress) {
        self.require_validator_range(config.min_validators, config.max_validators);
        self.sovereign_config().set(config);
        self.add_admin(admin);
    }

    #[only_admin]
    #[endpoint(updateConfig)]
    fn update_config(&self, new_config: SovereignConfig<Self::Api>) {
        self.require_config_set();
        self.require_validator_range(new_config.min_validators, new_config.max_validators);

        self.sovereign_config().set(new_config);
    }

    #[only_owner]
    #[endpoint(completeSetupPhase)]
    fn complete_setup_phase(&self, header_verifier_address: ManagedAddress) {
        if self.is_setup_phase_complete() {
            return;
        }

        self.require_config_set();
        // validator set in header verifier
        self.tx()
            .to(ToSelf)
            .typed(UserBuiltinProxy)
            .change_owner_address(&header_verifier_address)
            .sync_call();

        self.setup_phase_complete().set(true);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
