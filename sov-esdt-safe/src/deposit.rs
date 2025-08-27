multiversx_sc::imports!();
use structs::aliases::{EventPaymentTuple, OptionalValueTransferDataTuple};

#[multiversx_sc::module]
pub trait DepositModule:
    multiversx_sc_modules::pause::PauseModule
    + utils::UtilsModule
    + cross_chain::deposit_common::DepositCommonModule
    + cross_chain::execute_common::ExecuteCommonModule
    + cross_chain::storage::CrossChainStorage
    + custom_events::CustomEventsModule
{
    #[payable]
    #[endpoint]
    fn deposit(
        &self,
        to: ManagedAddress,
        opt_transfer_data: OptionalValueTransferDataTuple<Self::Api>,
    ) {
        self.deposit_common(to, opt_transfer_data, |payment| {
            self.process_payment(payment)
        });
    }

    fn process_payment(
        &self,
        payment: &EsdtTokenPayment<Self::Api>,
    ) -> EventPaymentTuple<Self::Api> {
        let current_sc_address = &self.blockchain().get_sc_address();
        self.burn_sovereign_token(payment);
        self.get_event_payment_token_data(current_sc_address, payment)
    }
}
