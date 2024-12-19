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
        require!(
            config.min_validators <= config.max_validators,
            "Invalid min/max validator numbers"
        );

        self.min_validators().set(config.min_validators);
        self.max_validators().set(config.max_validators);
        self.min_stake().set(config.min_stake);
        self.add_admin(admin);

        if let Some(additional_stake_required) = config.opt_additional_stake_required {
            self.additional_stake_required()
                .extend(&additional_stake_required);
        }
    }

    #[only_admin]
    fn update_config(&self, new_config: SovereignConfig<Self::Api>) {
        if !self.is_new_min_validators_value(new_config.min_validators) {
            self.min_validators().set(new_config.min_validators);
        }

        if !self.is_new_max_validators_value(new_config.max_validators) {
            self.max_validators().set(new_config.max_validators);
        }

        if !self.is_new_min_stake_value(&new_config.min_stake) {
            self.min_stake().set(new_config.min_stake);
        }

        if let Some(additional_stake_required) = new_config.opt_additional_stake_required {
            self.additional_stake_required()
                .extend(&additional_stake_required);
        }
    }

    #[only_owner]
    fn complete_setup_phase(&self, header_verifier_address: ManagedAddress) {
        if self.is_setup_phase_complete() {
            return;
        }

        self.require_config_set();
        // validator set in header verifier
        self.tx()
            .to(ESDTSystemSCAddress)
            .typed(UserBuiltinProxy)
            .change_owner_address(&header_verifier_address)
            .sync_call();

        self.setup_phase_complete().set(true);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
