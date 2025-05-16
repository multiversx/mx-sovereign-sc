use multiversx_sc::types::TokenIdentifier;
use proxies::chain_factory_proxy::ChainFactoryContractProxy;
use structs::configs::{EsdtSafeConfig, SovereignConfig};
use structs::fee::FeeStruct;

use crate::common::{self, utils::ScArray};
use crate::err_msg;

#[multiversx_sc::module]
pub trait UpdateConfigsModule: common::utils::UtilsModule + common::storage::StorageModule {
    #[endpoint(updateEsdtSafeConfig)]
    fn update_esdt_safe_config(&self, new_config: EsdtSafeConfig<Self::Api>) {
        let caller = self.blockchain().get_caller();

        self.require_phase_three_completed(&caller);

        self.tx()
            .to(self.get_chain_factory_address())
            .typed(ChainFactoryContractProxy)
            .update_esdt_safe_config(
                self.get_contract_address(&caller, ScArray::ESDTSafe),
                new_config,
            )
            .sync_call();
    }

    #[endpoint(updateSovereignConfig)]
    fn update_sovereign_config(&self, new_config: SovereignConfig<Self::Api>) {
        let blockchain_api = self.blockchain();
        let caller = blockchain_api.get_caller();

        self.require_phase_one_completed(&caller);

        self.tx()
            .to(self.get_chain_factory_address())
            .typed(ChainFactoryContractProxy)
            .update_sovereign_config(
                self.get_contract_address(&caller, ScArray::ChainConfig),
                new_config,
            )
            .sync_call();
    }

    #[endpoint(setFee)]
    fn set_fee(&self, new_fee: FeeStruct<Self::Api>) {
        let blockchain_api = self.blockchain();
        let caller = blockchain_api.get_caller();

        self.require_phase_four_completed(&caller);

        self.tx()
            .to(self.get_chain_factory_address())
            .typed(ChainFactoryContractProxy)
            .set_fee(
                self.get_contract_address(&caller, ScArray::FeeMarket),
                new_fee,
            )
            .sync_call();
    }

    #[endpoint(removeFee)]
    fn remove_fee(&self, token_id: TokenIdentifier<Self::Api>) {
        let blockchain_api = self.blockchain();
        let caller = blockchain_api.get_caller();

        self.require_phase_four_completed(&caller);

        self.tx()
            .to(self.get_chain_factory_address())
            .typed(ChainFactoryContractProxy)
            .remove_fee(
                self.get_contract_address(&caller, ScArray::FeeMarket),
                token_id,
            )
            .sync_call();
    }
}
