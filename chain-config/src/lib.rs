#![no_std]

use validator_rules::TokenIdAmountPair;

multiversx_sc::imports!();

pub const BLS_SIGNATURE_LEN: usize = 48;
pub type BlsSignature<M> = ManagedByteArray<M, BLS_SIGNATURE_LEN>;
// pub const MINIMUM_EGLD_STAKE_VALUE: u32 = 1000000000;

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

    #[endpoint(finishSetup)]
    fn finish_setup(&self) {
        self.require_caller_is_admin();

        let caller = self.blockchain().get_caller();
        self.remove_admin(caller);

        self.add_admin(self.sovereign_multisig_address().get());
    }

    #[payable("*")]
    #[endpoint(register)]
    fn register(
        &self,
        _egld_stake_value: StakeMultiArg<Self::Api>,
        // bls_authenticity: MultiValue2<ManagedBuffer, BlsSignature<Self::Api>>,
    ) {
        // query sovereign validator staking contract
        // check if genesis happened
        // let (bls_pub_key, bls_signature) = bls_authenticity.into_tuple();
        // let (_token_identifier, stake_amount) = egld_stake_value.into_tuple();
        //
        // require!(
        //     stake_amount >= MINIMUM_EGLD_STAKE_VALUE,
        //     "Staked eGLD minimum value has not been met"
        // );

        // self.registered_validators(bls_pub_key).set(bls_signature);
        // logEvent:
        // Identifier = register
        // Address = scAddress
        // Topics = address, blsKeys, eGLDStake, tokenStake
    }

    #[endpoint(unregister)]
    fn unregister(&self, _bls_pub_keys: MultiValueEncoded<ManagedBuffer>) {
        // sov system SC 
        // logEvent:
        //     Identifier = unRegister
        //     Address = scAddress
        //     Topics = address, blsKeys, eGLDStake, tokenStake

    }

    #[endpoint]
    fn upgrade(&self) {}

    #[storage_mapper("registeredValidators")]
    fn registered_validators(&self, bls_pub_key: ManagedBuffer) -> SingleValueMapper<BlsSignature<Self::Api>>;

    #[storage_mapper("sovereignMultiSigAddress")]
    fn sovereign_multisig_address(&self) -> SingleValueMapper<ManagedAddress>;
}
