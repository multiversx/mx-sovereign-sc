use transaction::{BatchId, PaymentsVec, Transaction, TxNonce};

multiversx_sc::imports!();

const NFT_AMOUNT: u32 = 1;

pub struct CheckMustRefundArgs<'a, M: ManagedTypeApi> {
    pub token: &'a EsdtTokenPayment<M>,
    pub roles: EsdtLocalRoleFlags,
    pub dest: &'a ManagedAddress<M>,
    pub batch_id: BatchId,
    pub tx_nonce: TxNonce,
    pub sc_address: &'a ManagedAddress<M>,
    pub sc_shard: u32,
}

#[multiversx_sc::module]
pub trait RefundModule:
    super::events::EventsModule
    + tx_batch_module::TxBatchModule
    + max_bridged_amount_module::MaxBridgedAmountModule
{
    fn check_must_refund(&self, args: CheckMustRefundArgs<Self::Api>) -> bool {
        let token_balance = self.blockchain().get_esdt_balance(
            args.sc_address,
            &args.token.token_identifier,
            args.token.token_nonce,
        );
        if token_balance < args.token.amount {
            if args.token.token_nonce == 0 {
                if !args.roles.has_role(&EsdtLocalRole::Mint) {
                    self.transfer_failed_invalid_token(args.batch_id, args.tx_nonce);

                    return true;
                }
            } else if !self.has_nft_roles(args.token, args.roles) {
                self.transfer_failed_invalid_token(args.batch_id, args.tx_nonce);

                return true;
            }
        }

        if self.is_above_max_amount(&args.token.token_identifier, &args.token.amount) {
            self.transfer_over_max_amount(args.batch_id, args.tx_nonce);

            return true;
        }

        if self.is_account_same_shard_frozen(args.sc_shard, args.dest, &args.token.token_identifier)
        {
            self.transfer_failed_frozen_destination_account(args.batch_id, args.tx_nonce);

            return true;
        }

        false
    }

    fn has_nft_roles(&self, payment: &EsdtTokenPayment, roles: EsdtLocalRoleFlags) -> bool {
        if !roles.has_role(&EsdtLocalRole::NftCreate) {
            return false;
        }

        if payment.amount > NFT_AMOUNT && !roles.has_role(&EsdtLocalRole::NftAddQuantity) {
            return false;
        }

        true
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
        let tx_nonce = self.get_and_save_next_tx_id();
        self.add_refund_transaction_event(tx_nonce, sov_tx.nonce);

        // invert from and to
        Transaction {
            block_nonce: self.blockchain().get_block_nonce(),
            nonce: tx_nonce,
            from: sov_tx.to,
            to: sov_tx.from,
            tokens: tokens_to_refund,
            token_data: ManagedVec::new(),
            opt_transfer_data: None,
            is_refund_tx: true,
        }
    }
}
