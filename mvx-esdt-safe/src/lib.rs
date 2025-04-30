#![no_std]

use error_messages::{NATIVE_TOKEN_NOT_REGISTERED, SETUP_PHASE_ALREADY_COMPLETED};

use multiversx_sc::imports::*;
use structs::configs::EsdtSafeConfig;

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
    + multiversx_sc_modules::only_admin::OnlyAdminModule
{
    #[init]
    fn init(
        &self,
        header_verifier_address: ManagedAddress,
        opt_config: OptionalValue<EsdtSafeConfig<Self::Api>>,
    ) {
        self.require_sc_address(&header_verifier_address);
        self.header_verifier_address().set(&header_verifier_address);
        self.admins().insert(self.blockchain().get_caller());

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

    #[only_admin]
    #[endpoint(updateConfiguration)]
    fn update_configuration(&self, new_config: EsdtSafeConfig<Self::Api>) {
        self.require_esdt_config_valid(&new_config);
        self.esdt_safe_config().set(new_config);
    }

    #[only_admin]
    #[endpoint(setFeeMarketAddress)]
    fn set_fee_market_address(&self, fee_market_address: ManagedAddress) {
        self.require_sc_address(&fee_market_address);
        self.fee_market_address().set(fee_market_address);
    }

    #[only_admin]
    #[endpoint(setMaxBridgedAmount)]
    fn set_max_bridged_amount(&self, token_id: TokenIdentifier, max_amount: BigUint) {
        self.max_bridged_amount(&token_id).set(&max_amount);
    }

    #[only_admin]
    #[endpoint(completSetupPhase)]
    fn complete_setup_phase(&self) {
        require!(
            !self.is_setup_phase_complete(),
            SETUP_PHASE_ALREADY_COMPLETED
        );

        // TODO:
        // require!(!self.native_token().is_empty(), NATIVE_TOKEN_NOT_REGISTERED);

        self.tx()
            .to(ToSelf)
            .typed(UserBuiltinProxy)
            .change_owner_address(&self.header_verifier_address().get())
            .sync_call();

        self.unpause_endpoint();

        self.setup_phase_complete().set(true);
    }
}
