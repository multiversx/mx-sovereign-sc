use transaction::{GasLimit, StolenFromFrameworkEsdtTokenData, Transaction, TransferData};

use crate::to_sovereign::events::DepositEvent;

multiversx_sc::imports!();

const MAX_USER_TX_GAS_LIMIT: GasLimit = 300_000_000;
const MAX_TRANSFERS_PER_TX: usize = 10;

#[multiversx_sc::module]
pub trait CreateTxModule:
    super::events::EventsModule
    + tx_batch_module::TxBatchModule
    + max_bridged_amount_module::MaxBridgedAmountModule
    + multiversx_sc_modules::pause::PauseModule
{
    /// Create an MultiversX -> Sovereign transaction.
    #[payable("*")]
    #[endpoint]
    fn deposit(
        &self,
        to: ManagedAddress,
        opt_gas_limit: OptionalValue<GasLimit>,
        opt_function: OptionalValue<ManagedBuffer>,
        opt_args: OptionalValue<MultiValueEncoded<ManagedBuffer>>,
    ) {
        require!(self.not_paused(), "Cannot create transaction while paused");

        let payments = self.call_value().all_esdt_transfers().clone_value();
        require!(!payments.is_empty(), "Nothing to transfer");
        require!(payments.len() <= MAX_TRANSFERS_PER_TX, "Too many tokens");

        let opt_transfer_data = if let OptionalValue::Some(gas_limit) = opt_gas_limit {
            require!(gas_limit <= MAX_USER_TX_GAS_LIMIT, "Gas limit too high");
            require!(opt_function.is_some(), "Must provide function name");

            let args_list = opt_args.into_option().unwrap_or(MultiValueEncoded::new());
            let args = args_list.to_vec();

            OptionalValue::Some(TransferData {
                gas_limit,
                function: opt_function.into_option().unwrap(),
                args,
            })
        } else {
            OptionalValue::None
        };

        let own_sc_address = self.blockchain().get_sc_address();
        let mut all_token_data = ManagedVec::new();
        let mut event_payments = MultiValueEncoded::new();
        for payment in &payments {
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

            event_payments.push(
                (
                    payment.token_identifier,
                    payment.token_nonce,
                    payment.amount,
                )
                    .into(),
            );
        }

        let caller = self.blockchain().get_caller();
        let block_nonce = self.blockchain().get_block_nonce();
        let tx_nonce = self.get_and_save_next_tx_id();

        self.deposit_event(
            &to,
            &event_payments,
            DepositEvent::from(tx_nonce, &opt_transfer_data),
        );

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
        let _ = self.add_to_batch(tx);
    }
}
