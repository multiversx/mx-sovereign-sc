use multiversx_sc::imports::*;
pub const UNBOND_PERIOD: u64 = 10;
pub const DELEGATE_ENDPOINT: &str = "delegate";
pub const UNDELEGATE_ENDPOINT: &str = "unDelegate";
pub const CLAIM_REWARDS_ENDPOINT: &str = "claimRewards";

use crate::common::{self, storage::Epoch};

#[multiversx_sc::module]
pub trait DelegationModule: common::storage::CommonStorageModule {
    #[payable("EGLD")]
    #[endpoint(stake)]
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
                self.delegated_value(&caller)
                    .update(|value| *value -= egld_amount_to_unstake);
                self.undelegate_epoch(caller)
                    .set(current_epoch + UNBOND_PERIOD);
            }
            _ => sc_panic!("There was an error at delegating"),
        }
    }

    #[endpoint(claimRewardsFromDelegation)]
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
}
