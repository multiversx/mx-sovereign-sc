#![no_std]

use error_messages::{ERROR_AT_ENCODING, SETUP_PHASE_ALREADY_COMPLETED};

use multiversx_sc::imports::*;
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
    + utils::UtilsModule
    + setup_phase::SetupPhaseModule
{
    #[init]
    fn init(&self, opt_config: OptionalValue<EsdtSafeConfig<Self::Api>>) {
        let new_config = match opt_config {
            OptionalValue::Some(cfg) => {
                if let Some(error_message) = self.is_esdt_safe_config_valid(&cfg) {
                    sc_panic!(error_message);
                }
                cfg
            }
            OptionalValue::None => EsdtSafeConfig::default_config(),
        };

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
        self.require_setup_complete();

        let config_hash = new_config.generate_hash();
        require!(!config_hash.is_empty(), ERROR_AT_ENCODING);

        self.lock_operation_hash_wrapper(&hash_of_hashes, &config_hash);

        if let Some(error_message) = self.is_esdt_safe_config_valid(&new_config) {
            self.complete_operation(
                &hash_of_hashes,
                &config_hash,
                Some(ManagedBuffer::from(error_message)),
            );
        } else {
            self.esdt_safe_config().set(new_config);
        }

        self.complete_operation(&hash_of_hashes, &config_hash, None);
    }

    #[only_owner]
    #[endpoint(setFeeMarketAddress)]
    fn set_fee_market_address(&self, fee_market_address: ManagedAddress) {
        self.require_sc_address(&fee_market_address);
        self.fee_market_address().set(fee_market_address);
    }

    #[only_owner]
    #[endpoint(completeSetupPhase)]
    fn complete_setup_phase(&self) {
        require!(
            !self.is_setup_phase_complete(),
            SETUP_PHASE_ALREADY_COMPLETED
        );

        // TODO:
        // require!(!self.native_token().is_empty(), NATIVE_TOKEN_NOT_REGISTERED);

        self.unpause_endpoint();

        self.setup_phase_complete().set(true);
    }
}
