use crate::{events, validator_rules, StakeMultiArg};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait BlsKeysModule: validator_rules::ValidatorRulesModule + events::EventsModule {
    #[payable("*")]
    #[endpoint(register)]
    fn register(
        &self,
        pub_bls_keys: MultiValueEncoded<ManagedBuffer>,
        egld_stake_value: StakeMultiArg<Self::Api>,
    ) {
        self.require_bls_keys_length_limits(pub_bls_keys.len());

        let (_, stake_amount) = egld_stake_value.into_tuple();
        self.require_min_stake(stake_amount);

        for bls_key in pub_bls_keys.clone() {
            let has_stake = self.has_stake_in_validator_sc(&bls_key);
            require!(has_stake, "The validator with the {} BLS key does not have any stake in the SovereignValidatorSC", bls_key);

            // let delegated_value = self.get_delegated_value(validator_id)

            self.require_bls_key_whitelist(&bls_key);

            self.bls_keys().insert(bls_key);
        }

        let current_sc_address = self.blockchain().get_sc_address();
        self.register_event(&current_sc_address, &pub_bls_keys);
    }
}