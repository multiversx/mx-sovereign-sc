use multiversx_sc::{api::const_handles::BIG_INT_CONST_ZERO, imports::*};
pub const UNBOND_PERIOD: u64 = 10;
pub const DELEGATE_ENDPOINT: &str = "delegate";
pub const UNDELEGATE_ENDPOINT: &str = "unDelegate";

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
            .raw_call(DELEGATE_ENDPOINT)
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
                self.delegated_value(caller.clone()).set(egld_amount);
                self.egld_token_supply()
                    .update(|value| *value += egld_amount)
            }
            _ => sc_panic!("There was an error at delegating"),
        }
    }

    #[endpoint(unStake)]
    fn unstake(&self, contract_name: ManagedBuffer, egld_amount_to_unstake: BigUint) {
        let caller = self.blockchain().get_caller();
        let total_egld_deposit = self.delegated_value(caller.clone()).get();
        require!(
            total_egld_deposit > BIG_INT_CONST_ZERO,
            "The user has not deposited any EGLD"
        );

        let current_epoch = self.blockchain().get_block_epoch();
        let delegation_contract_address = self.delegation_addresses(&contract_name).get();

        let mut args: ManagedArgBuffer<Self::Api> = ManagedArgBuffer::new();
        args.push_arg(&egld_amount_to_unstake);

        require!(
            egld_amount_to_unstake < total_egld_deposit,
            "The value to unstake is greater than the deposited amount"
        );

        self.tx()
            .to(delegation_contract_address)
            .raw_call(ManagedBuffer::from(UNDELEGATE_ENDPOINT))
            .argument(&args)
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
                self.delegated_value(caller.clone())
                    .update(|value| *value -= egld_amount_to_unstake);
                self.undelegate_epoch(caller)
                    .set(current_epoch + UNBOND_PERIOD);
            }
            _ => sc_panic!("There was an error at delegating"),
        }
    }

    // TODO: Could use a Enum for each endpoint name
    // fn call_delegation_contract_endpoint(&self, endpoint_name: ManagedBuffer)
}
