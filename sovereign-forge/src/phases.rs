use core::ops::Deref;

use crate::{common, err_msg};

#[multiversx_sc::module]
pub trait PhasesModule: common::utils::UtilsModule + common::storage::StorageModule {
    #[payable("EGLD")]
    #[endpoint(deployPhaseOne)]
    fn deploy_phase_one(&self) {
        let call_value = self.call_value().egld_value();
        self.require_correct_deploy_cost(call_value.deref());

        let chain_id = self.generate_chain_id();
    }
}
