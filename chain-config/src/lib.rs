#![no_std]

use multiversx_sc_modules::only_admin;
use transaction::{SovereignConfig, StakeArgs};

multiversx_sc::imports!();

pub mod validator_rules;

#[multiversx_sc::contract]
pub trait ChainConfigContract:
    validator_rules::ValidatorRulesModule + only_admin::OnlyAdminModule + setup_phase::SetupPhaseModule
{
    #[init]
    fn init(
        &self,
        min_validators: u64,
        max_validators: u64,
        min_stake: BigUint,
        admin: ManagedAddress,
        additional_stake_required: MultiValueEncoded<StakeArgs<Self::Api>>,
    ) {
        require!(
            min_validators <= max_validators,
            "Invalid min/max validator numbers"
        );

        self.min_validators().set(min_validators);
        self.max_validators().set(max_validators);
        self.min_stake().set(min_stake);
        self.add_admin(admin);
        self.additional_stake_required()
            .extend(additional_stake_required);
    }

    #[only_admin]
    fn update_config(&self, new_config: SovereignConfig<Self::Api>) {
        if let Some(min_validators) = new_config.opt_min_validators {
            self.min_validators().set(min_validators);
        }
        if let Some(max_validators) = new_config.opt_max_validators {
            self.max_validators().set(max_validators);
        }
        if let Some(min_stake) = new_config.opt_min_stake {
            self.min_stake().set(min_stake);
        }
        if let Some(additional_stake_required) = new_config.opt_additional_stake_required {
            self.additional_stake_required()
                .extend(&additional_stake_required);
        }
    }

    #[only_owner]
    fn complete_setup_phase(&self) {
        if self.is_setup_phase_complete() {
            return;
        }

        self.require_config_set();
        // validator set in header verifier
        // change ownership to header-verifier
        self.setup_phase_complete().set(true);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
