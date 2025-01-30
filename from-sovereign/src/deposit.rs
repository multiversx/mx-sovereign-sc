multiversx_sc::imports!();
use multiversx_sc::storage::StorageKey;
use operation::aliases::{ExtractedFeeResult, OptionalValueTransferDataTuple};

const MAX_TRANSFERS_PER_TX: usize = 10;

#[multiversx_sc::module]
pub trait DepositModule:
    multiversx_sc_modules::pause::PauseModule + utils::UtilsModule + cross_chain::CrossChainCommon
{
    #[payable]
    #[endpoint]
    fn deposit(
        &self,
        to: ManagedAddress,
        opt_transfer_data: OptionalValueTransferDataTuple<Self::Api>,
    ) {
        require!(self.not_paused(), "Cannot create transaction while paused");

        let (fees_payment, payments) = self.check_and_extract_fee().into_tuple();
        require!(!payments.is_empty(), "Nothing to transfer");
        require!(payments.len() <= MAX_TRANSFERS_PER_TX, "Too many tokens");

        let own_sc_address = self.blockchain().get_sc_address();
    }

    fn check_and_extract_fee(&self) -> ExtractedFeeResult<Self::Api> {
        let payments = self.call_value().all_esdt_transfers().clone();

        require!(!payments.is_empty(), "Nothing to transfer");
        require!(payments.len() <= MAX_TRANSFERS_PER_TX, "Too many tokens");

        let fee_market_address = self.fee_market_address().get();
        let fee_enabled_mapper = SingleValueMapper::new_from_address(
            fee_market_address.clone(),
            StorageKey::from("feeEnabledFlag"),
        )
        .get();

        let opt_transfer_data = if fee_enabled_mapper {
            OptionalValue::Some(self.pop_first_payment(payments.clone()).0)
        } else {
            OptionalValue::None
        };

        MultiValue2::from((opt_transfer_data, payments))
    }
}
