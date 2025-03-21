#![no_std]

use error_messages::INVALID_MIN_MAX_VALIDATOR_NUMBERS;
use validator_rules::TokenIdAmountPair;

multiversx_sc::imports!();

pub mod bridge;
pub mod validator_rules;

pub type StakeMultiArg<M> = MultiValue2<TokenIdentifier<M>, BigUint<M>>;

#[multiversx_sc::contract]
pub trait ChainConfigContract:
    bridge::BridgeModule
    + validator_rules::ValidatorRulesModule
    + multiversx_sc_modules::only_admin::OnlyAdminModule
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
            INVALID_MIN_MAX_VALIDATOR_NUMBERS
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
