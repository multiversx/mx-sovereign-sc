#![no_std]
#![allow(non_snake_case)]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use transaction::{transaction_status::TransactionStatus, Transaction, TransferData};

const DEFAULT_MAX_TX_BATCH_SIZE: usize = 10;
const DEFAULT_MAX_TX_BATCH_BLOCK_DURATION: u64 = 100; // ~10 minutes
const MAX_TRANSFERS_PER_TX: usize = 10;

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem)]
pub struct NonceAmountPair<M: ManagedTypeApi> {
    pub nonce: u64,
    pub amount: BigUint<M>,
}

#[multiversx_sc::contract]
pub trait EsdtSafe:
    token_module::TokenModule
    + tx_batch_module::TxBatchModule
    + max_bridged_amount_module::MaxBridgedAmountModule
    + multiversx_sc_modules::pause::PauseModule
{
    /// sovereign_tx_gas_limit - The gas limit that will be used for transactions on the Sovereign side.
    /// In case of SC gas limits, this value is provided by the user
    /// Will be used to compute the fees for the transfer
    #[init]
    fn init(&self, sovereign_tx_gas_limit: BigUint) {
        self.sovereign_tx_gas_limit().set(&sovereign_tx_gas_limit);

        self.max_tx_batch_size()
            .set_if_empty(DEFAULT_MAX_TX_BATCH_SIZE);
        self.max_tx_batch_block_duration()
            .set_if_empty(DEFAULT_MAX_TX_BATCH_BLOCK_DURATION);

        // batch ID 0 is considered invalid
        self.first_batch_id().set_if_empty(1);
        self.last_batch_id().set_if_empty(1);

        self.set_paused(true);
    }

    /// Sets the statuses for the transactions, after they were executed on the Sovereign side.
    ///
    /// Only TransactionStatus::Executed (3) and TransactionStatus::Rejected (4) values are allowed.
    /// Number of provided statuses must be equal to number of transactions in the batch.
    #[only_owner]
    #[endpoint(setTransactionBatchStatus)]
    fn set_transaction_batch_status(
        &self,
        batch_id: u64,
        tx_statuses: MultiValueEncoded<TransactionStatus>,
    ) {
        let first_batch_id = self.first_batch_id().get();
        require!(
            batch_id == first_batch_id,
            "Batches must be processed in order"
        );

        let mut tx_batch = self.pending_batches(batch_id);
        require!(
            tx_batch.len() == tx_statuses.len(),
            "Invalid number of statuses provided"
        );

        for (tx, tx_status) in tx_batch.iter().zip(tx_statuses.to_vec().iter()) {
            // Since tokens don't exist in the EsdtSafe in the case of a refund transaction
            // we have no tokens to burn, nor to refund
            if tx.is_refund_tx {
                continue;
            }

            match tx_status {
                TransactionStatus::Executed => {
                    // local burn role might be removed while tx is executed
                    // tokens will remain locked forever in that case
                    // otherwise, the whole batch would fail
                    for token in &tx.tokens {
                        if self.is_local_role_set(&token.token_identifier, &EsdtLocalRole::Burn) {
                            self.send().esdt_local_burn(
                                &token.token_identifier,
                                token.token_nonce,
                                &token.amount,
                            )
                        }
                    }
                }
                TransactionStatus::Rejected => {
                    for token in &tx.tokens {
                        self.mark_refund(&tx.from, &token);
                    }
                }
                _ => {
                    sc_panic!("Transaction status may only be set to Executed or Rejected");
                }
            }

            self.set_status_event(batch_id, tx.nonce, tx_status);
        }

        self.clear_first_batch(&mut tx_batch);
    }

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

    // endpoints

    /// Create an Elrond -> Sovereign transaction.
    #[payable("*")]
    #[endpoint(createTransaction)]
    fn create_transaction(
        &self,
        to: ManagedAddress,
        opt_transfer_data: OptionalValue<TransferData<Self::Api>>,
    ) {
        require!(self.not_paused(), "Cannot create transaction while paused");

        let payments = self.call_value().all_esdt_transfers().clone_value();
        require!(!payments.is_empty(), "Nothing to transfer");
        require!(payments.len() <= MAX_TRANSFERS_PER_TX, "Too many tokens");

        let own_sc_address = self.blockchain().get_sc_address();
        let mut all_token_data = ManagedVec::new();
        for payment in &payments {
            self.require_token_in_whitelist(&payment.token_identifier);
            self.require_below_max_amount(&payment.token_identifier, &payment.amount);

            if payment.token_nonce > 0 {
                let current_token_data = self.blockchain().get_esdt_token_data(
                    &own_sc_address,
                    &payment.token_identifier,
                    payment.token_nonce,
                );
                all_token_data.push(Some(current_token_data.into()));
            } else {
                all_token_data.push(None);
            }
        }

        let caller = self.blockchain().get_caller();
        let block_nonce = self.blockchain().get_block_nonce();
        let tx_nonce = self.get_and_save_next_tx_id();
        let tx = Transaction {
            block_nonce,
            nonce: tx_nonce,
            from: caller,
            to,
            tokens: payments,
            token_data: all_token_data,
            opt_transfer_data: opt_transfer_data.into_option(),
            is_refund_tx: false,
        };

        let batch_id = self.add_to_batch(tx);
        self.create_transaction_event(batch_id, tx_nonce);
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

    // private

    fn mark_refund(&self, to: &ManagedAddress, token: &EsdtTokenPayment) {
        self.refund_amount(to, &token.token_identifier)
            .update(|refund| {
                refund.push(NonceAmountPair {
                    nonce: token.token_nonce,
                    amount: token.amount.clone(),
                });
            });
    }

    // events

    #[event("createTransactionEvent")]
    fn create_transaction_event(&self, #[indexed] batch_id: u64, #[indexed] tx_id: u64);

    #[event("addRefundTransactionEvent")]
    fn add_refund_transaction_event(
        &self,
        #[indexed] batch_id: u64,
        #[indexed] tx_id: u64,
        #[indexed] original_tx_id: u64,
    );

    #[event("setStatusEvent")]
    fn set_status_event(
        &self,
        #[indexed] batch_id: u64,
        #[indexed] tx_id: u64,
        #[indexed] tx_status: TransactionStatus,
    );

    // storage

    #[storage_mapper("refundAmount")]
    fn refund_amount(
        &self,
        address: &ManagedAddress,
        token_id: &TokenIdentifier,
    ) -> SingleValueMapper<ManagedVec<NonceAmountPair<Self::Api>>>;

    #[view(getSovereignTxGasLimit)]
    #[storage_mapper("sovereignTxGasLimit")]
    fn sovereign_tx_gas_limit(&self) -> SingleValueMapper<BigUint>;
}
