#![no_std]

use error_messages::SETUP_PHASE_ALREADY_COMPLETED;

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
    + cross_chain::events::EventsModule
    + cross_chain::storage::CrossChainStorage
    + cross_chain::execute_common::ExecuteCommonModule
    + multiversx_sc_modules::pause::PauseModule
    + utils::UtilsModule
    + setup_phase::SetupPhaseModule
{
    #[init]
    fn init(
        &self,
        header_verifier_address: ManagedAddress,
        opt_config: OptionalValue<EsdtSafeConfig<Self::Api>>,
    ) {
        self.require_sc_address(&header_verifier_address);
        self.header_verifier_address().set(&header_verifier_address);

        self.esdt_safe_config().set(
            opt_config
                .into_option()
                .inspect(|config| self.require_esdt_config_valid(config))
                .unwrap_or_else(EsdtSafeConfig::default_config),
        );

        self.set_paused(true);
    }

    #[upgrade]
    fn upgrade(&self) {}

    #[only_owner]
    #[endpoint(updateEsdtSafeConfigSetupPhase)]
    fn update_esdt_safe_config_during_setup_phase(&self, new_config: EsdtSafeConfig<Self::Api>) {
        self.require_esdt_config_valid(&new_config);
        self.esdt_safe_config().set(new_config);
    }

    #[endpoint(updateEsdtSafeConfig)]
    fn update_esdt_safe_config(
        &self,
        hash_of_hashes: ManagedBuffer,
        new_config: EsdtSafeConfig<Self::Api>,
    ) {
        let opt_hash = if self.is_setup_phase_complete() {
            Some(new_config.generate_hash())
        } else {
            None
        };

        if let Some(ref config_hash) = opt_hash {
            self.lock_operation_hash(config_hash, &hash_of_hashes);
        }

        self.require_esdt_config_valid(&new_config);
        self.esdt_safe_config().set(new_config);

        if let Some(config_hash) = opt_hash {
            self.remove_executed_hash(&hash_of_hashes, &config_hash);
        }
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
