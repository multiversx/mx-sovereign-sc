use multiversx_sc::types::{MultiValueEncoded, TokenIdentifier};
use proxies::chain_factory_proxy::ChainFactoryContractProxy;
use structs::configs::{EsdtSafeConfig, SovereignConfig};
use structs::fee::FeeStruct;
use structs::forge::ScArray;

use crate::{err_msg, forge_common};

#[multiversx_sc::module]
pub trait UpdateConfigsModule:
    common_utils::CommonUtilsModule
    + forge_common::storage::StorageModule
    + forge_common::forge_utils::ForgeUtilsModule
    + custom_events::CustomEventsModule
{
    #[endpoint(updateEsdtSafeConfig)]
    fn update_esdt_safe_config(&self, new_config: EsdtSafeConfig<Self::Api>) {
        let caller = self.blockchain().get_caller();

        self.require_phase_two_completed(&caller);

        self.tx()
            .to(self.get_chain_factory_address())
            .typed(ChainFactoryContractProxy)
            .update_esdt_safe_config(
                self.get_contract_address(&caller, ScArray::ESDTSafe),
                new_config,
            )
            .transfer_execute();
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
            .transfer_execute();
    }

    #[endpoint(setFee)]
    fn set_fee(&self, new_fee: FeeStruct<Self::Api>) {
        let blockchain_api = self.blockchain();
        let caller = blockchain_api.get_caller();

        self.require_phase_three_completed(&caller);

        self.tx()
            .to(self.get_chain_factory_address())
            .typed(ChainFactoryContractProxy)
            .set_fee(
                self.get_contract_address(&caller, ScArray::FeeMarket),
                new_fee,
            )
            .transfer_execute();
    }

    #[endpoint(removeFee)]
    fn remove_fee(&self, token_id: TokenIdentifier<Self::Api>) {
        let blockchain_api = self.blockchain();
        let caller = blockchain_api.get_caller();

        self.require_phase_three_completed(&caller);

        self.tx()
            .to(self.get_chain_factory_address())
            .typed(ChainFactoryContractProxy)
            .remove_fee(
                self.get_contract_address(&caller, ScArray::FeeMarket),
                token_id,
            )
            .transfer_execute();
    }

    #[endpoint(addUsersToWhitelist)]
    fn add_users_to_whitelist(&self, users: MultiValueEncoded<ManagedAddress>) {
        let blockchain_api = self.blockchain();
        let caller = blockchain_api.get_caller();

        self.require_phase_three_completed(&caller);

        self.tx()
            .to(self.get_chain_factory_address())
            .typed(ChainFactoryContractProxy)
            .add_users_to_whitelist(
                self.get_contract_address(&caller, ScArray::FeeMarket),
                users,
            )
            .transfer_execute();
    }

    #[endpoint(removeUsersFromWhitelist)]
    fn remove_users_from_whitelist(&self, users: MultiValueEncoded<ManagedAddress>) {
        let blockchain_api = self.blockchain();
        let caller = blockchain_api.get_caller();

        self.require_phase_three_completed(&caller);

        self.tx()
            .to(self.get_chain_factory_address())
            .typed(ChainFactoryContractProxy)
            .remove_users_from_whitelist(
                self.get_contract_address(&caller, ScArray::FeeMarket),
                users,
            )
            .transfer_execute();
    }

    #[endpoint(setTokenBurnMechanism)]
    fn set_token_burn_mechanism(&self, token_id: EgldOrEsdtTokenIdentifier) {
        let blockchain_api = self.blockchain();
        let caller = blockchain_api.get_caller();

        self.require_phase_three_completed(&caller);

        self.tx()
            .to(self.get_chain_factory_address())
            .typed(ChainFactoryContractProxy)
            .set_token_burn_mechanism(
                self.get_contract_address(&caller, ScArray::ESDTSafe),
                token_id,
            )
            .transfer_execute();
    }

    #[endpoint(setTokenLockMechanism)]
    fn set_token_lock_mechanism(&self, token_id: EgldOrEsdtTokenIdentifier) {
        let blockchain_api = self.blockchain();
        let caller = blockchain_api.get_caller();

        self.require_phase_three_completed(&caller);

        self.tx()
            .to(self.get_chain_factory_address())
            .typed(ChainFactoryContractProxy)
            .set_token_lock_mechanism(
                self.get_contract_address(&caller, ScArray::ESDTSafe),
                token_id,
            )
            .transfer_execute();
    }
}
