use crate::{validator_rules, StakeMultiArg};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait BlsKeysModule: validator_rules::ValidatorRulesModule {
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

        for bls_key in pub_bls_keys {
            require!(self.has_stake_in_validator_sc(&bls_key), "The validator with the {} BLS key does not have any stake in the SovereignValidatorSC", bls_key);

            self.require_bls_key_whitelist(&bls_key);

            self.bls_keys().insert(bls_key);
        }
    }
}
