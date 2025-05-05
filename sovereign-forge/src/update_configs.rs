use multiversx_sc::require;
use proxies::chain_factory_proxy::ChainFactoryContractProxy;
use structs::configs::{EsdtSafeConfig, SovereignConfig};

use crate::common::{self, utils::ScArray};
use crate::err_msg;

#[multiversx_sc::module]
pub trait UpdateConfigsModule: common::utils::UtilsModule + common::storage::StorageModule {
    #[endpoint(updateEsdtSafeConfig)]
    fn update_esdt_safe_config(&self, new_config: EsdtSafeConfig<Self::Api>) {
        let caller = self.blockchain().get_caller();

        self.require_phase_three_completed(&caller);

        require!(
            !self.is_contract_deployed(&caller, ScArray::FeeMarket),
            "The Fee-Market SC is already deployed"
        );

        let esdt_safe_address = self.get_contract_address(&caller, ScArray::ESDTSafe);

        self.tx()
            .to(self.get_chain_factory_address())
            .typed(ChainFactoryContractProxy)
            .update_esdt_safe_config(esdt_safe_address, new_config)
            .sync_call();
    }

    #[endpoint(updateSovereignConfig)]
    fn update_sovereign_config(&self, new_config: SovereignConfig<Self::Api>) {
        let blockchain_api = self.blockchain();
        let caller = blockchain_api.get_caller();

        self.require_phase_one_completed(&caller);
        require!(
            !self.is_contract_deployed(&caller, ScArray::HeaderVerifier),
            "The Header-Verifier contract is already deployed"
        );

        let chain_config_address = self.get_contract_address(&caller, ScArray::ChainConfig);

        self.tx()
            .to(self.get_chain_factory_address())
            .typed(ChainFactoryContractProxy)
            .update_sovereign_config(chain_config_address, new_config)
            .sync_call();
    }
}
