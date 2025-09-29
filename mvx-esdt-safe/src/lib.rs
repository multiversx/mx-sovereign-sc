#![no_std]

use error_messages::{
    ERROR_AT_GENERATING_OPERATION_HASH, FEE_MARKET_NOT_SET, NATIVE_TOKEN_NOT_REGISTERED, SETUP_PHASE_ALREADY_COMPLETED, SETUP_PHASE_NOT_COMPLETED
};

use multiversx_sc::imports::*;
use multiversx_sc_modules::only_admin;
use structs::{configs::EsdtSafeConfig, generate_hash::GenerateHash};

pub mod bridging_mechanism;
pub mod deposit;
pub mod execute;
pub mod register_token;

#[multiversx_sc::contract]
pub trait MvxEsdtSafe:
    deposit::DepositModule
    + cross_chain::LibCommon
    + execute::ExecuteModule
    + register_token::RegisterTokenModule
    + bridging_mechanism::BridgingMechanism
    + cross_chain::deposit_common::DepositCommonModule
    + custom_events::CustomEventsModule
    + cross_chain::storage::CrossChainStorage
    + cross_chain::execute_common::ExecuteCommonModule
    + multiversx_sc_modules::pause::PauseModule
    + common_utils::CommonUtilsModule
    + setup_phase::SetupPhaseModule
    + only_admin::OnlyAdminModule
{
    #[init]
    fn init(
        &self,
        sovereign_owner: ManagedAddress,
        sov_token_prefix: ManagedBuffer,
        opt_config: OptionalValue<EsdtSafeConfig<Self::Api>>,
    ) {
        self.validate_chain_id(&sov_token_prefix);

        self.sov_token_prefix().set(sov_token_prefix);

        let new_config = match opt_config {
            OptionalValue::Some(cfg) => {
                if let Some(error_message) = self.is_esdt_safe_config_valid(&cfg) {
                    sc_panic!(error_message);
                }
                cfg
            }
            OptionalValue::None => EsdtSafeConfig::default_config(),
        };

        self.add_admin(sovereign_owner);

        self.esdt_safe_config().set(new_config);

        self.set_paused(true);
    }

    #[upgrade]
    fn upgrade(&self) {}

    #[only_owner]
    #[endpoint(updateEsdtSafeConfigSetupPhase)]
    fn update_esdt_safe_config_during_setup_phase(&self, new_config: EsdtSafeConfig<Self::Api>) {
        require!(
            !self.is_setup_phase_complete(),
            SETUP_PHASE_ALREADY_COMPLETED
        );

        if let Some(error_message) = self.is_esdt_safe_config_valid(&new_config) {
            sc_panic!(error_message);
        }

        self.esdt_safe_config().set(new_config);
    }

    #[endpoint(updateEsdtSafeConfig)]
    fn update_esdt_safe_config(
        &self,
        hash_of_hashes: ManagedBuffer,
        new_config: EsdtSafeConfig<Self::Api>,
    ) {
        let config_hash = new_config.generate_hash();
        if config_hash.is_empty() {
            self.complete_operation(
                &hash_of_hashes,
                &config_hash,
                Some(ERROR_AT_GENERATING_OPERATION_HASH.into()),
            );
            return;
        }
        if !self.is_setup_phase_complete() {
            self.complete_operation(
                &hash_of_hashes,
                &config_hash,
                Some(SETUP_PHASE_NOT_COMPLETED.into()),
            );
            return;
        }
        if let Some(lock_operation_error) =
            self.lock_operation_hash_wrapper(&hash_of_hashes, &config_hash)
        {
            self.complete_operation(&hash_of_hashes, &config_hash, Some(lock_operation_error));
            return;
        }
        if let Some(error_message) = self.is_esdt_safe_config_valid(&new_config) {
            self.complete_operation(
                &hash_of_hashes,
                &config_hash,
                Some(ManagedBuffer::from(error_message)),
            );
            return;
        } else {
            self.esdt_safe_config().set(new_config);
            self.complete_operation(&hash_of_hashes, &config_hash, None);
        }
    }

    #[only_owner]
    #[endpoint(setFeeMarketAddress)]
    fn set_fee_market_address(&self, fee_market_address: ManagedAddress) {
        require!(
            !self.is_setup_phase_complete(),
            SETUP_PHASE_ALREADY_COMPLETED
        );
        self.require_sc_address(&fee_market_address);
        self.fee_market_address().set(fee_market_address);
    }

    #[only_owner]
    #[endpoint(completeSetupPhase)]
    fn complete_setup_phase(&self) {
        if self.is_setup_phase_complete() {
            return;
        }

        //TODO: Uncomment this after fixing the issue with the native token
        require!(!self.native_token().is_empty(), NATIVE_TOKEN_NOT_REGISTERED);
        require!(!self.fee_market_address().is_empty(), FEE_MARKET_NOT_SET);

        self.unpause_endpoint();
        self.remove_admin(self.admins().get_by_index(1));
        self.setup_phase_complete().set(true);
    }
}
