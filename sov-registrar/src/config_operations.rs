use structs::configs::{
    EsdtSafeConfig, SovereignConfig, UpdateEsdtSafeConfigOperation,
    UpdateRegistrationStatusOperation, UpdateSovereignConfigOperation,
};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait ConfigOperationsModule:
    tx_nonce::TxNonceModule
    + custom_events::CustomEventsModule
    + fee_common::helpers::FeeCommonHelpersModule
    + fee_common::storage::FeeCommonStorageModule
    + utils::UtilsModule
{
    #[endpoint(registerUpdateSovereignConfig)]
    fn register_update_sovereign_config(&self, sovereign_config: SovereignConfig<Self::Api>) {
        self.update_sovereign_config_event(UpdateSovereignConfigOperation {
            sovereign_config,
            nonce: self.get_and_save_next_tx_id(),
        });
    }

    #[endpoint(registerUpdateRegistrationStatus)]
    fn register_update_registration_status(&self, registration_status: u8) {
        self.update_registration_status_event(UpdateRegistrationStatusOperation {
            registration_status,
            nonce: self.get_and_save_next_tx_id(),
        });
    }

    #[endpoint(registerUpdateEsdtSafeConfig)]
    fn register_update_esdt_safe_config(&self, esdt_safe_config: EsdtSafeConfig<Self::Api>) {
        self.update_esdt_safe_config_event(UpdateEsdtSafeConfigOperation {
            esdt_safe_config,
            nonce: self.get_and_save_next_tx_id(),
        });
    }
}
