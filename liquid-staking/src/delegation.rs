use multiversx_sc::imports::*;
pub const UNBOND_PERIOD: u64 = 10;

use crate::common::{self, storage::Epoch};

#[multiversx_sc::module]
pub trait DelegationModule: common::storage::CommonStorageModule {
    #[payable("ELGD")]
    #[endpoint(stake)]
    fn stake(&self) {
        let caller = self.blockchain().get_caller();
        let egld_amount = self.call_value().egld_value().clone_value();
        let delegation_contract_address = self.delegation_address().get();
        let delegation_endpoint = ManagedBuffer::from("delegate");

        self.tx()
            .to(delegation_contract_address)
            .raw_call(delegation_endpoint)
            .egld(&egld_amount)
            .with_callback(DelegationModule::callbacks(self).stake_callback(&caller, &egld_amount))
            .call_and_exit();
    }

    #[endpoint(unStake)]
    fn unstake(&self, egld_amount_to_unstake: BigUint) {
        let caller = self.blockchain().get_caller();
        let current_epoch = self.blockchain().get_block_epoch();
        let total_egld_deposit = self.delegated_value(caller.clone()).get();
        let delegation_contract_address = self.delegation_address().get();
        let undelegate_endpoint = ManagedBuffer::from("unDelegate");

        let mut args: ManagedArgBuffer<Self::Api> = ManagedArgBuffer::new();
        args.push_arg(&egld_amount_to_unstake);

        require!(
            egld_amount_to_unstake < total_egld_deposit,
            "The value to unstake is greater than the deposited amount"
        );

        self.tx()
            .to(delegation_contract_address)
            .raw_call(undelegate_endpoint)
            .argument(&args)
            .with_callback(DelegationModule::callbacks(self).unstake_callback(
                &caller,
                &egld_amount_to_unstake,
                current_epoch,
            ))
            .call_and_exit();
    }

    // TODO: Could use a Enum for each endpoint name
    // fn call_delegation_contract_endpoint(&self, endpoint_name: ManagedBuffer)

    #[callback]
    fn stake_callback(
        &self,
        caller: &ManagedAddress,
        egld_amount: &BigUint,
        #[call_result] result: ManagedAsyncCallResult<()>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(()) => {
                self.delegated_value(caller.clone()).set(egld_amount);
                self.egld_token_supply()
                    .update(|value| *value += egld_amount)
            }
            _ => sc_panic!("There was an error at delegating"),
        }
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
                self.delegated_value(caller.clone())
                    .update(|value| *value -= egld_amount_to_unstake);
                self.undelegate_epoch(caller)
                    .set(current_epoch + UNBOND_PERIOD);
            }
            _ => sc_panic!("There was an error at delegating"),
        }
    }
}
