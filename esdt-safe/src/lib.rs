#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use transaction::GasLimit;
use tx_batch_module::FIRST_BATCH_ID;

const DEFAULT_MAX_TX_BATCH_SIZE: usize = 10;
const DEFAULT_MAX_TX_BATCH_BLOCK_DURATION: u64 = 100; // ~10 minutes

pub mod from_sovereign;
pub mod to_sovereign;

#[multiversx_sc::contract]
pub trait EsdtSafe:
    to_sovereign::create_tx::CreateTxModule
    + to_sovereign::events::EventsModule
    + to_sovereign::refund::RefundModule
    + to_sovereign::set_tx_status::SetTxStatusModule
    + bls_signature::BlsSignatureModule
    + from_sovereign::events::EventsModule
    + from_sovereign::refund::RefundModule
    + from_sovereign::token_mapping::TokenMappingModule
    + from_sovereign::transfer_tokens::TransferTokensModule
    + token_module::TokenModule
    + tx_batch_module::TxBatchModule
    + max_bridged_amount_module::MaxBridgedAmountModule
    + multiversx_sc_modules::pause::PauseModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    /// sovereign_tx_gas_limit - The gas limit that will be used for transactions on the Sovereign side.
    /// In case of SC gas limits, this value is provided by the user
    /// Will be used to compute the fees for the transfer
    #[init]
    fn init(
        &self,
        sovereign_tx_gas_limit: GasLimit,
        multi_transfer_sc_address: ManagedAddress,
        min_valid_signers: u32,
        signers: MultiValueEncoded<ManagedAddress>,
    ) {
        require!(
            self.blockchain()
                .is_smart_contract(&multi_transfer_sc_address),
            "Invalid SC address"
        );

        self.sovereign_tx_gas_limit().set(sovereign_tx_gas_limit);
        self.multi_transfer_sc_address()
            .set(multi_transfer_sc_address);

        self.max_tx_batch_size().set(DEFAULT_MAX_TX_BATCH_SIZE);
        self.max_tx_batch_block_duration()
            .set(DEFAULT_MAX_TX_BATCH_BLOCK_DURATION);

        // batch ID 0 is considered invalid
        self.first_batch_id().set(FIRST_BATCH_ID);
        self.last_batch_id().set(FIRST_BATCH_ID);

        self.set_min_valid_signers(min_valid_signers);
        self.add_signers(signers);

        self.set_paused(true);
    }

    #[endpoint]
    fn upgrade(&self) {}
}
