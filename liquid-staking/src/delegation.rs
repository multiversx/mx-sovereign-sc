use multiversx_sc::imports::*;
pub const UNBOND_PERIOD: u64 = 10;
pub const DELEGATE_ENDPOINT: &[u8] = b"delegate";
pub const UNDELEGATE_ENDPOINT: &[u8] = b"unDelegate";
pub const CLAIM_REWARDS_ENDPOINT: &[u8] = b"claimRewards";

use crate::common::{
    self,
    storage::{BlsKey, ChainId, Epoch},
};

#[multiversx_sc::module]
pub trait DelegationModule: common::storage::CommonStorageModule {
    #[payable("EGLD")]
    #[endpoint]
    fn stake(&self, contract_name: ManagedBuffer) {
        let caller = self.blockchain().get_caller();
        let egld_amount = self.call_value().egld_value().clone_value();
        let delegation_contract_address = self.delegation_addresses(&contract_name).get();

        self.tx()
            .to(delegation_contract_address)
            .raw_call(ManagedBuffer::from(DELEGATE_ENDPOINT))
            .egld(&egld_amount)
            .callback(DelegationModule::callbacks(self).stake_callback(&caller, &egld_amount))
            .async_call_and_exit();
    }

    #[callback]
    fn stake_callback(
        &self,
        caller: &ManagedAddress,
        egld_amount: &BigUint,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                self.delegated_value(caller).set(egld_amount);
                self.egld_token_supply()
                    .update(|value| *value += egld_amount)
            }
            _ => sc_panic!("There was an error at delegating"),
        }
    }

    #[endpoint(unStake)]
    fn unstake(&self, contract_name: ManagedBuffer, egld_amount_to_unstake: BigUint) {
        let caller = self.blockchain().get_caller();
        self.require_caller_has_stake(&caller);

        let current_epoch = self.blockchain().get_block_epoch();
        let delegation_contract_address = self.delegation_addresses(&contract_name).get();
        let total_egld_deposit = self.delegated_value(&caller).get();

        require!(
            egld_amount_to_unstake <= total_egld_deposit,
            "The value to unstake is greater than the deposited amount"
        );

        self.tx()
            .to(delegation_contract_address)
            .raw_call(ManagedBuffer::from(UNDELEGATE_ENDPOINT))
            .argument(&egld_amount_to_unstake)
            .callback(DelegationModule::callbacks(self).unstake_callback(
                &caller,
                &egld_amount_to_unstake,
                current_epoch,
            ))
            .async_call_and_exit();
    }

    #[callback]
    fn unstake_callback(
        &self,
        caller: &ManagedAddress,
        egld_amount_to_unstake: &BigUint,
        current_epoch: Epoch,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                self.delegated_value(caller)
                    .update(|value| *value -= egld_amount_to_unstake);
                self.undelegate_epoch(caller)
                    .set(current_epoch + UNBOND_PERIOD);
            }
            _ => sc_panic!("There was an error at delegating"),
        }
    }

    #[endpoint]
    fn claim_rewards_from_delegation(&self, contracts: MultiValueEncoded<ManagedBuffer>) {
        let caller = self.blockchain().get_caller();
        self.require_caller_has_stake(&caller);

        for delegation_contract in contracts {
            let delegation_mapper = self.delegation_addresses(&delegation_contract);
            if !delegation_mapper.is_empty() {
                let delegation_address = delegation_mapper.get();
                self.tx()
                    .to(delegation_address)
                    .raw_call(CLAIM_REWARDS_ENDPOINT)
                    .callback(DelegationModule::callbacks(self).claim_rewards_from_delegation_cb())
                    .async_call_and_exit();
            }
        }
    }

    #[callback]
    fn claim_rewards_from_delegation_cb(
        &self,
        // egld_amount: &BigUint,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {}
            _ => sc_panic!("There was an error at claiming rewards from the delegation contract"),
        }
    }

    // NOTE: Should this also add to the map ?
    #[endpoint]
    fn slash_validator(
        &self,
        validator_address: ManagedAddress,
        bls_key: BlsKey<Self::Api>,
        value_to_slash: BigUint,
    ) {
        let caller = self.blockchain().get_caller();
        self.require_caller_to_be_header_verifier(&caller);
        self.require_bls_key_to_be_registered(&bls_key);

        require!(
            !self.validator_bls_key_address_map(&bls_key).is_empty(),
            "There is no associated address to the given BLS key"
        );

        self.require_caller_has_stake(&validator_address);

        require!(value_to_slash > 0, "You can't slash a value of 0 eGLD");

        let delegated_value = self.delegated_value(&validator_address).get();
        require!(
            delegated_value >= value_to_slash,
            "The slash value can't be greater than the total delegated amount"
        );

        self.delegated_value(&validator_address)
            .update(|value| *value -= &value_to_slash);
    }

    #[payable("EGLD")]
    #[endpoint(lockForSovereignChain)]
    fn lock_for_sovereign_chain(&self, chain_id: ChainId<Self::Api>) {
        let call_value = self.call_value().egld_value().clone_value();

        require!(call_value > 0, "No value send to lock");
        self.locked_supply(chain_id)
            .update(|supply| *supply += call_value);

        // lock amount with ChainConfigSC
    }

    #[endpoint]
    fn claim_rewards(&self) {}
}
