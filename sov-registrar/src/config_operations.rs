use structs::configs::{EsdtSafeConfig, SovereignConfig};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait ConfigOperationsModule:
    tx_nonce::TxNonceModule
    + custom_events::CustomEventsModule
    + fee_common::helpers::FeeCommonHelpersModule
    + fee_common::storage::FeeCommonStorageModule
    + utils::UtilsModule
{
    #[endpoint(updateSovereignConfig)]
    fn update_sovereign_config(&self, sovereign_config: SovereignConfig<Self::Api>) {
        self.update_sovereign_config_event(sovereign_config, self.get_and_save_next_tx_id());
    }

    #[endpoint(updateRegistrationStatus)]
    fn update_registration_status(&self, registration_status: u8) {
        self.update_registration_status_event(registration_status, self.get_and_save_next_tx_id());
    }

    #[endpoint(updateEsdtSafeConfig)]
    fn update_esdt_safe_config(&self, esdt_safe_config: EsdtSafeConfig<Self::Api>) {
        self.update_esdt_safe_config_event(esdt_safe_config, self.get_and_save_next_tx_id());
    }
}
