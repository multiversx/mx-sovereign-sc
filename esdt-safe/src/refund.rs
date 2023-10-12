use transaction::Transaction;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem)]
pub struct NonceAmountPair<M: ManagedTypeApi> {
    pub nonce: u64,
    pub amount: BigUint<M>,
}

#[multiversx_sc::module]
pub trait RefundModule:
    crate::events::EventsModule
    + token_module::TokenModule
    + tx_batch_module::TxBatchModule
    + max_bridged_amount_module::MaxBridgedAmountModule
{
    /// Converts failed Sovereign -> Elrond transactions to Elrond -> Sovereign transaction.
    /// This is done every now and then to refund the tokens.
    ///
    /// As with normal Elrond -> Sovereign transactions, a part of the tokens will be
    /// subtracted to pay for the fees
    #[only_owner]
    #[endpoint(addRefundBatch)]
    fn add_refund_batch(&self, refund_transactions: ManagedVec<Transaction<Self::Api>>) {
        let block_nonce = self.blockchain().get_block_nonce();
        let mut new_transactions = ManagedVec::new();
        let mut original_tx_nonces = ManagedVec::<Self::Api, u64>::new();

        for refund_tx in &refund_transactions {
            let tx_nonce = self.get_and_save_next_tx_id();

            // "from" and "to" are inverted, since this was initially a Sovereign -> Elrond tx
            let new_tx = Transaction {
                block_nonce,
                nonce: tx_nonce,
                from: refund_tx.to,
                to: refund_tx.from,
                tokens: refund_tx.tokens,
                token_data: refund_tx.token_data,
                opt_transfer_data: None,
                is_refund_tx: true,
            };
            new_transactions.push(new_tx);
            original_tx_nonces.push(refund_tx.nonce);
        }

        let batch_ids = self.add_multiple_tx_to_batch(&new_transactions);
        for (i, tx) in new_transactions.iter().enumerate() {
            let batch_id = batch_ids.get(i);
            let original_tx_nonce = original_tx_nonces.get(i);

            self.add_refund_transaction_event(batch_id, tx.nonce, original_tx_nonce);
        }
    }

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
