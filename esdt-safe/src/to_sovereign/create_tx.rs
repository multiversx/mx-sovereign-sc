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
    + token_whitelist::TokenWhitelistModule
    + bls_signature::BlsSignatureModule
    + setup_phase::SetupPhaseModule
    + utils::UtilsModule
    + multiversx_sc_modules::pause::PauseModule
{
    /// Create an Elrond -> Sovereign transaction.
    #[payable("*")]
    #[endpoint]
    fn deposit(
        &self,
        to: ManagedAddress,
        opt_transfer_data: OptionalValue<TransferData<Self::Api>>,
    ) {
        require!(self.not_paused(), "Cannot create transaction while paused");

        // TODO: Take fees, send remaining amount to user

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
            self.require_below_max_amount(&payment.token_identifier, &payment.amount);
            self.require_token_not_blacklisted(&payment.token_identifier);

            // TODO: When adding fees, take whitelist into account

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

        self.deposit_event(
            &to,
            &payments,
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
