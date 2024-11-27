#![no_std]

use multiversx_sc_modules::only_admin;
use transaction::StakeMultiArg;
use validator_rules::TokenIdAmountPair;

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
        additional_stake_required: MultiValueEncoded<StakeMultiArg<Self::Api>>,
    ) {
        require!(
            min_validators <= max_validators,
            "Invalid min/max validator numbers"
        );

        let mut additional_stake_vec = ManagedVec::new();
        for multi_value in additional_stake_required {
            let (token_id, amount) = multi_value.into_tuple();
            let value = TokenIdAmountPair { token_id, amount };

            additional_stake_vec.push(value);
        }

        self.min_validators().set(min_validators);
        self.max_validators().set(max_validators);
        self.min_stake().set(min_stake);
        self.add_admin(admin);
        self.additional_stake_required().set(additional_stake_vec);
    }

    #[only_admin]
    fn update_config(
        &self,
        opt_min_validators: Option<u64>,
        opt_max_validators: Option<u64>,
        opt_min_stake: Option<BigUint>,
        opt_additional_stake_required: Option<MultiValueEncoded<StakeMultiArg<Self::Api>>>,
    ) {
        if let Some(min_validators) = opt_min_validators {
            self.min_validators().set(min_validators);
        }
        if let Some(max_validators) = opt_max_validators {
            self.max_validators().set(max_validators);
        }
        if let Some(min_stake) = opt_min_stake {
            self.min_stake().set(min_stake);
        }
        if let Some(additional_stake_required) = opt_additional_stake_required {
            let mut additional_stake_vec = ManagedVec::new();
            for multi_value in additional_stake_required {
                let (token_id, amount) = multi_value.into_tuple();
                let value = TokenIdAmountPair { token_id, amount };

                additional_stake_vec.push(value);
            }
            self.additional_stake_required().set(additional_stake_vec);
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
        // update setup_phase_complete
    }

    #[upgrade]
    fn upgrade(&self) {}
}
