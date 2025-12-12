use crate::err_msg;
use multiversx_sc::{imports::IgnoreValue, sc_panic, types::ManagedAsyncCallResult};
use structs::forge::{ContractInfo, ScArray};

use crate::forge_common::{forge_utils, storage};

#[multiversx_sc::module]
pub trait ForgeCallbackModule:
    forge_utils::ForgeUtilsModule
    + storage::StorageModule
    + common_utils::CommonUtilsModule
    + custom_events::CustomEventsModule
{
    #[promises_callback]
    fn setup_phase(
        &self,
        sovereign_owner: &ManagedAddress,
        #[call_result] result: ManagedAsyncCallResult<IgnoreValue>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(_) => {
                self.sovereign_setup_phase(&self.sovereigns_mapper(sovereign_owner).get())
                    .set(true);
            }
            ManagedAsyncCallResult::Err(result) => {
                sc_panic!(result.err_msg);
            }
        }
    }

    #[promises_callback]
    fn register_deployed_contract(
        &self,
        chain_id: &ManagedBuffer,
        sc_id: ScArray,
        #[call_result] result: ManagedAsyncCallResult<ManagedAddress>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(sc_address) => {
                let new_contract_info = ContractInfo::new(sc_id, sc_address);

                self.sovereign_deployed_contracts(chain_id)
                    .insert(new_contract_info);
            }
            ManagedAsyncCallResult::Err(call_err) => {
                sc_panic!(call_err.err_msg);
            }
        }
    }

    #[promises_callback]
    fn update_configs(&self, #[call_result] result: ManagedAsyncCallResult<IgnoreValue>) {
        match result {
            ManagedAsyncCallResult::Ok(_) => {}
            ManagedAsyncCallResult::Err(err) => sc_panic!("{}", err.err_msg),
        }
    }
}
