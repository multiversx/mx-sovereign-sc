use transaction::{GasLimit, StolenFromFrameworkEsdtTokenData, Transaction, TransferData};

multiversx_sc::imports!();

const MAX_USER_TX_GAS_LIMIT: GasLimit = 300_000_000;
const MAX_TRANSFERS_PER_TX: usize = 10;

#[multiversx_sc::module]
pub trait CreateTxModule:
    crate::events::EventsModule
    + token_module::TokenModule
    + tx_batch_module::TxBatchModule
    + max_bridged_amount_module::MaxBridgedAmountModule
    + multiversx_sc_modules::pause::PauseModule
{
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

        if let OptionalValue::Some(transfer_data) = &opt_transfer_data {
            require!(
                transfer_data.gas_limit <= MAX_USER_TX_GAS_LIMIT,
                "Gas limit too high"
            );
        }

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
                all_token_data.push(current_token_data.into());
            } else {
                all_token_data.push(StolenFromFrameworkEsdtTokenData::default());
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

        let default_gas_cost = self.sovereign_tx_gas_limit().get();
        let batch_id = self.add_to_batch(tx, default_gas_cost);
        self.create_transaction_event(batch_id, tx_nonce);
    }

    #[view(getSovereignTxGasLimit)]
    #[storage_mapper("sovereignTxGasLimit")]
    fn sovereign_tx_gas_limit(&self) -> SingleValueMapper<GasLimit>;
}
