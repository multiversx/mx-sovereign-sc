multiversx_sc::imports!();
use error_messages::CALLER_IS_BLACKLISTED;
use structs::aliases::{EventPaymentTuple, OptionalValueTransferDataTuple};

#[multiversx_sc::module]
pub trait DepositModule:
    crate::bridging_mechanism::BridgingMechanism
    + common_utils::CommonUtilsModule
    + setup_phase::SetupPhaseModule
    + cross_chain::deposit_common::DepositCommonModule
    + cross_chain::execute_common::ExecuteCommonModule
    + cross_chain::storage::CrossChainStorage
    + custom_events::CustomEventsModule
    + multiversx_sc_modules::pause::PauseModule
{
    #[payable]
    #[endpoint]
    fn deposit(
        &self,
        to: ManagedAddress,
        opt_transfer_data: OptionalValueTransferDataTuple<Self::Api>,
    ) {
        self.require_setup_complete();
        self.require_caller_not_blacklisted();
        self.deposit_common(to, opt_transfer_data, |payment| {
            self.process_payment(payment)
        });
    }

    fn require_caller_not_blacklisted(&self) {
        let caller = self.blockchain().get_caller();
        require!(
            !self.deposit_callers_blacklist().contains(&caller),
            CALLER_IS_BLACKLISTED
        );
    }

    fn process_payment(
        &self,
        payment: &EgldOrEsdtTokenPayment<Self::Api>,
    ) -> EventPaymentTuple<Self::Api> {
        let token_identifier = payment.token_identifier.clone();
        let mut token_data = if token_identifier.is_egld() {
            EsdtTokenData::default()
        } else {
            let esdt_id = token_identifier.clone().unwrap_esdt();
            self.blockchain().get_esdt_token_data(
                &self.blockchain().get_sc_address(),
                &esdt_id,
                payment.token_nonce,
            )
        };

        token_data.amount = payment.amount.clone();

        let token_mapper = self.multiversx_to_sovereign_token_id_mapper(&token_identifier);
        if !token_mapper.is_empty() || self.is_native_token(&token_identifier) {
            let sov_token_id = token_mapper.get();
            let sov_token_nonce = self.burn_mainchain_token(
                &token_identifier,
                payment.token_nonce,
                &payment.amount,
                &token_data.token_type,
                &sov_token_id,
            );
            MultiValue3::from((sov_token_id.clone(), sov_token_nonce, token_data))
        } else {
            if self.is_fungible(&token_data.token_type)
                && self.burn_mechanism_tokens().contains(&token_identifier)
            {
                let esdt_id = token_identifier.clone().unwrap_esdt();
                self.tx()
                    .to(ToSelf)
                    .typed(UserBuiltinProxy)
                    .esdt_local_burn(esdt_id, payment.token_nonce, payment.amount.clone())
                    .sync_call();

                self.deposited_tokens_amount(&token_identifier)
                    .update(|amount| *amount += payment.amount.clone());
            }

            MultiValue3::from((token_identifier.clone(), payment.token_nonce, token_data))
        }
    }

    #[view(getDepositCallersBlacklist)]
    #[storage_mapper("depositCallersBlacklist")]
    fn deposit_callers_blacklist(&self) -> UnorderedSetMapper<ManagedAddress>;
}
