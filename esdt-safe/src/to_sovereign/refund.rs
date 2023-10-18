multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem)]
pub struct NonceAmountPair<M: ManagedTypeApi> {
    pub nonce: u64,
    pub amount: BigUint<M>,
}

#[multiversx_sc::module]
pub trait RefundModule:
    super::events::EventsModule
    + token_module::TokenModule
    + tx_batch_module::TxBatchModule
    + max_bridged_amount_module::MaxBridgedAmountModule
{
    /// Claim funds for failed Elrond -> Sovereign transactions.
    /// These are not sent automatically to prevent the contract getting stuck.
    /// For example, if the receiver is a SC, a frozen account, etc.
    #[endpoint(claimRefund)]
    fn claim_refund(&self, token_id: TokenIdentifier) -> ManagedVec<EsdtTokenPayment> {
        let caller = self.blockchain().get_caller();
        let refund_amounts = self.refund_amount(&caller, &token_id).take();
        require!(!refund_amounts.is_empty(), "Nothing to refund");

        let mut output_payments = ManagedVec::new();
        for nonce_amount_pair in &refund_amounts {
            output_payments.push(EsdtTokenPayment::new(
                token_id.clone(),
                nonce_amount_pair.nonce,
                nonce_amount_pair.amount,
            ));
        }

        self.send().direct_multi(&caller, &output_payments);

        output_payments
    }

    /// Query function that lists all refund amounts for a user.
    /// Useful for knowing which token IDs to pass to the claimRefund endpoint.
    #[view(getRefundAmounts)]
    fn get_refund_amounts(
        &self,
        address: ManagedAddress,
    ) -> MultiValueEncoded<MultiValue3<TokenIdentifier, u64, BigUint>> {
        let mut refund_amounts = MultiValueEncoded::new();
        for token_id in self.token_whitelist().iter() {
            let nonce_amount_pairs = self.refund_amount(&address, &token_id).get();
            for nonce_amount_pair in &nonce_amount_pairs {
                refund_amounts.push(
                    (
                        token_id.clone(),
                        nonce_amount_pair.nonce,
                        nonce_amount_pair.amount,
                    )
                        .into(),
                );
            }
        }

        refund_amounts
    }

    fn mark_refund(&self, to: &ManagedAddress, token: &EsdtTokenPayment) {
        self.refund_amount(to, &token.token_identifier)
            .update(|refund| {
                refund.push(NonceAmountPair {
                    nonce: token.token_nonce,
                    amount: token.amount.clone(),
                });
            });
    }

    #[storage_mapper("refundAmount")]
    fn refund_amount(
        &self,
        address: &ManagedAddress,
        token_id: &TokenIdentifier,
    ) -> SingleValueMapper<ManagedVec<NonceAmountPair<Self::Api>>>;
}
