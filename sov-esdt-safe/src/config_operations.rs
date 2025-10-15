use structs::configs::{EsdtSafeConfig, SovereignConfig};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait ConfigOperationsModule:
    tx_nonce::TxNonceModule
    + custom_events::CustomEventsModule
    + fee_common::helpers::FeeCommonHelpersModule
    + fee_common::storage::FeeCommonStorageModule
    + common_utils::CommonUtilsModule
{
    #[only_owner]
    #[endpoint(updateSovereignConfig)]
    fn update_sovereign_config(&self, sovereign_config: SovereignConfig<Self::Api>) {
        self.update_sovereign_config_event(sovereign_config, self.get_and_save_next_tx_id());
    }

    #[only_owner]
    #[endpoint(updateEsdtSafeConfig)]
    fn update_esdt_safe_config(&self, esdt_safe_config: EsdtSafeConfig<Self::Api>) {
        self.update_esdt_safe_config_event(esdt_safe_config, self.get_and_save_next_tx_id());
    }

    #[only_owner]
    #[endpoint(setTokenBurnMechanism)]
    fn set_token_burn_mechanism(&self, token_id: EgldOrEsdtTokenIdentifier<Self::Api>) {
        self.set_token_burn_mechanism_event(token_id, self.get_and_save_next_tx_id());
    }

    #[only_owner]
    #[endpoint(setTokenLockMechanism)]
    fn set_token_lock_mechanism(&self, token_id: EgldOrEsdtTokenIdentifier<Self::Api>) {
        self.set_token_lock_mechanism_event(token_id, self.get_and_save_next_tx_id());
    }
}
