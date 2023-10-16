use transaction::{BatchId, PaymentsVec, Transaction, TxBatchSplitInFields, TxNonce};

multiversx_sc::imports!();

const NFT_AMOUNT: u32 = 1;

#[multiversx_sc::module]
pub trait RefundModule:
    crate::events::EventsModule
    + tx_batch_module::TxBatchModule
    + max_bridged_amount_module::MaxBridgedAmountModule
{
    #[only_owner]
    #[endpoint(getAndClearFirstRefundBatch)]
    fn get_and_clear_first_refund_batch(&self) -> OptionalValue<TxBatchSplitInFields<Self::Api>> {
        let opt_current_batch = self.get_first_batch_any_status();
        if matches!(opt_current_batch, OptionalValue::Some(_)) {
            let first_batch_id = self.first_batch_id().get();
            let mut first_batch = self.pending_batches(first_batch_id);

            self.clear_first_batch(&mut first_batch);
        }

        opt_current_batch
    }

    fn check_must_refund(
        &self,
        token: &EsdtTokenPayment,
        dest: &ManagedAddress,
        batch_id: BatchId,
        tx_nonce: TxNonce,
        sc_shard: u32,
    ) -> bool {
        if token.token_nonce == 0 {
            if !self.is_local_role_set(&token.token_identifier, &EsdtLocalRole::Mint) {
                self.transfer_failed_invalid_token(batch_id, tx_nonce);

                return true;
            }
        } else if !self.has_nft_roles(token) {
            self.transfer_failed_invalid_token(batch_id, tx_nonce);

            return true;
        }

        if self.is_above_max_amount(&token.token_identifier, &token.amount) {
            self.transfer_over_max_amount(batch_id, tx_nonce);

            return true;
        }

        if self.is_account_same_shard_frozen(sc_shard, dest, &token.token_identifier) {
            self.transfer_failed_frozen_destination_account(batch_id, tx_nonce);

            return true;
        }

        false
    }

    fn has_nft_roles(&self, payment: &EsdtTokenPayment) -> bool {
        if !self.is_local_role_set(&payment.token_identifier, &EsdtLocalRole::NftCreate) {
            return false;
        }

        if payment.amount > NFT_AMOUNT
            && !self.is_local_role_set(&payment.token_identifier, &EsdtLocalRole::NftAddQuantity)
        {
            return false;
        }

        true
    }

    fn is_local_role_set(&self, token_id: &TokenIdentifier, role: &EsdtLocalRole) -> bool {
        let roles = self.blockchain().get_esdt_local_roles(token_id);

        roles.has_role(role)
    }

    fn is_account_same_shard_frozen(
        &self,
        sc_shard: u32,
        dest_address: &ManagedAddress,
        token_id: &TokenIdentifier,
    ) -> bool {
        let dest_shard = self.blockchain().get_shard_of_address(dest_address);
        if sc_shard != dest_shard {
            return false;
        }

        let token_data = self
            .blockchain()
            .get_esdt_token_data(dest_address, token_id, 0);
        token_data.frozen
    }

    fn convert_to_refund_tx(
        &self,
        sov_tx: Transaction<Self::Api>,
        tokens_to_refund: PaymentsVec<Self::Api>,
    ) -> Transaction<Self::Api> {
        Transaction {
            block_nonce: self.blockchain().get_block_nonce(),
            nonce: sov_tx.nonce,
            from: sov_tx.from,
            to: sov_tx.to,
            tokens: tokens_to_refund,
            token_data: ManagedVec::new(),
            opt_transfer_data: None,
            is_refund_tx: true,
        }
    }
}
