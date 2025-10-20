use error_messages::DEPLOY_COST_IS_ZERO;
use multiversx_sc::{
    imports::OptionalValue,
    require,
    types::{MultiValueEncoded, TokenIdentifier},
};
use proxies::chain_factory_proxy::ChainFactoryContractProxy;
use structs::configs::{EsdtSafeConfig, SovereignConfig};
use structs::fee::FeeStruct;
use structs::forge::ScArray;
use structs::{UPDATE_CONFIGS_CALLBACK_GAS, UPDATE_CONFIGS_GAS};

use crate::err_msg;
use crate::forge_common;
use crate::forge_common::callbacks::{self, CallbackProxy};

#[multiversx_sc::module]
pub trait UpdateConfigsModule:
    common_utils::CommonUtilsModule
    + forge_common::storage::StorageModule
    + forge_common::forge_utils::ForgeUtilsModule
    + custom_events::CustomEventsModule
    + callbacks::ForgeCallbackModule
{
    #[endpoint(updateEsdtSafeConfig)]
    fn update_esdt_safe_config(&self, new_config: EsdtSafeConfig<Self::Api>) {
        let caller = self.blockchain().get_caller();

        self.require_phase_two_completed(&caller);

        self.tx()
            .to(self.get_chain_factory_address(&caller))
            .typed(ChainFactoryContractProxy)
            .update_esdt_safe_config(
                self.get_contract_address(&caller, ScArray::ESDTSafe),
                new_config,
            )
            .gas(UPDATE_CONFIGS_GAS)
            .callback(self.callbacks().update_configs())
            .gas_for_callback(UPDATE_CONFIGS_CALLBACK_GAS)
            .register_promise();
    }

    #[endpoint(updateSovereignConfig)]
    fn update_sovereign_config(&self, new_config: SovereignConfig<Self::Api>) {
        let blockchain_api = self.blockchain();
        let caller = blockchain_api.get_caller();

        self.require_phase_one_completed(&caller);

        self.tx()
            .to(self.get_chain_factory_address(&caller))
            .typed(ChainFactoryContractProxy)
            .update_sovereign_config(
                self.get_contract_address(&caller, ScArray::ChainConfig),
                new_config,
            )
            .gas(UPDATE_CONFIGS_GAS)
            .callback(self.callbacks().update_configs())
            .gas_for_callback(UPDATE_CONFIGS_CALLBACK_GAS)
            .register_promise();
    }

    #[endpoint(setFee)]
    fn set_fee(&self, new_fee: FeeStruct<Self::Api>) {
        let blockchain_api = self.blockchain();
        let caller = blockchain_api.get_caller();

        self.require_phase_three_completed(&caller);

        self.tx()
            .to(self.get_chain_factory_address(&caller))
            .typed(ChainFactoryContractProxy)
            .set_fee(
                self.get_contract_address(&caller, ScArray::FeeMarket),
                new_fee,
            )
            .gas(UPDATE_CONFIGS_GAS)
            .callback(self.callbacks().update_configs())
            .gas_for_callback(UPDATE_CONFIGS_CALLBACK_GAS)
            .register_promise();
    }

    #[endpoint(removeFee)]
    fn remove_fee(&self, token_id: TokenIdentifier<Self::Api>) {
        let blockchain_api = self.blockchain();
        let caller = blockchain_api.get_caller();

        self.require_phase_three_completed(&caller);

        self.tx()
            .to(self.get_chain_factory_address(&caller))
            .typed(ChainFactoryContractProxy)
            .remove_fee(
                self.get_contract_address(&caller, ScArray::FeeMarket),
                token_id,
            )
            .gas(UPDATE_CONFIGS_GAS)
            .callback(self.callbacks().update_configs())
            .gas_for_callback(UPDATE_CONFIGS_CALLBACK_GAS)
            .register_promise();
    }

    #[endpoint(addUsersToWhitelist)]
    fn add_users_to_whitelist(&self, users: MultiValueEncoded<ManagedAddress>) {
        let blockchain_api = self.blockchain();
        let caller = blockchain_api.get_caller();

        self.require_phase_three_completed(&caller);

        self.tx()
            .to(self.get_chain_factory_address(&caller))
            .typed(ChainFactoryContractProxy)
            .add_users_to_whitelist(
                self.get_contract_address(&caller, ScArray::FeeMarket),
                users,
            )
            .gas(UPDATE_CONFIGS_GAS)
            .callback(self.callbacks().update_configs())
            .gas_for_callback(UPDATE_CONFIGS_CALLBACK_GAS)
            .register_promise();
    }

    #[endpoint(removeUsersFromWhitelist)]
    fn remove_users_from_whitelist(&self, users: MultiValueEncoded<ManagedAddress>) {
        let blockchain_api = self.blockchain();
        let caller = blockchain_api.get_caller();

        self.require_phase_three_completed(&caller);

        self.tx()
            .to(self.get_chain_factory_address(&caller))
            .typed(ChainFactoryContractProxy)
            .remove_users_from_whitelist(
                self.get_contract_address(&caller, ScArray::FeeMarket),
                users,
            )
            .gas(UPDATE_CONFIGS_GAS)
            .callback(self.callbacks().update_configs())
            .gas_for_callback(UPDATE_CONFIGS_CALLBACK_GAS)
            .register_promise();
    }

    #[endpoint(setTokenBurnMechanism)]
    fn set_token_burn_mechanism(&self, token_id: EgldOrEsdtTokenIdentifier) {
        let blockchain_api = self.blockchain();
        let caller = blockchain_api.get_caller();

        self.require_phase_two_completed(&caller);

        self.tx()
            .to(self.get_chain_factory_address(&caller))
            .typed(ChainFactoryContractProxy)
            .set_token_burn_mechanism(
                self.get_contract_address(&caller, ScArray::ESDTSafe),
                token_id,
            )
            .gas(UPDATE_CONFIGS_GAS)
            .callback(self.callbacks().update_configs())
            .gas_for_callback(UPDATE_CONFIGS_CALLBACK_GAS)
            .register_promise();
    }

    #[endpoint(setTokenLockMechanism)]
    fn set_token_lock_mechanism(&self, token_id: EgldOrEsdtTokenIdentifier) {
        let blockchain_api = self.blockchain();
        let caller = blockchain_api.get_caller();

        self.require_phase_two_completed(&caller);

        self.tx()
            .to(self.get_chain_factory_address(&caller))
            .typed(ChainFactoryContractProxy)
            .set_token_lock_mechanism(
                self.get_contract_address(&caller, ScArray::ESDTSafe),
                token_id,
            )
            .gas(UPDATE_CONFIGS_GAS)
            .callback(self.callbacks().update_configs())
            .gas_for_callback(UPDATE_CONFIGS_CALLBACK_GAS)
            .register_promise();
    }

    #[only_owner]
    #[endpoint(updateDeployCost)]
    fn update_deploy_cost(&self, opt_deploy_cost: OptionalValue<BigUint>) {
        match opt_deploy_cost {
            OptionalValue::Some(deploy_cost) => {
                require!(deploy_cost > 0, DEPLOY_COST_IS_ZERO);
                self.deploy_cost().set(deploy_cost);
            }
            OptionalValue::None => {
                self.deploy_cost().clear();
            }
        }
    }
}
