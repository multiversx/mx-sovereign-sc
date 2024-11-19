#![no_std]

use multiversx_sc_modules::only_admin;
use transaction::StakeMultiArg;
use validator_rules::TokenIdAmountPair;

multiversx_sc::imports!();

pub mod bridge;
pub mod validator_rules;

#[multiversx_sc::contract]
pub trait ChainConfigContract:
    bridge::BridgeModule + validator_rules::ValidatorRulesModule + only_admin::OnlyAdminModule
{
    #[init]
    fn init(
        &self,
        min_validators: usize,
        max_validators: usize,
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

    #[upgrade]
    fn upgrade(&self) {}
}
