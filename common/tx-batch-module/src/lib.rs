#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub use batch_status::BatchStatus;
use transaction::{
    BatchId, GasLimit, Transaction, TxBatchSplitInFields, TxNonce, MIN_BLOCKS_FOR_FINALITY,
};
use tx_batch_mapper::TxBatchMapper;

pub const FIRST_BATCH_ID: BatchId = 1;
pub const MAX_GAS_LIMIT_PER_BATCH: GasLimit = 500_000_000;

pub mod batch_status;
pub mod tx_batch_mapper;

#[multiversx_sc::module]
pub trait TxBatchModule {
    // endpoints - owner-only

    #[only_owner]
    #[endpoint(setMaxTxBatchSize)]
    fn set_max_tx_batch_size(&self, new_max_tx_batch_size: usize) {
        require!(
            new_max_tx_batch_size > 0,
            "Max tx batch size must be more than 0"
        );

        self.max_tx_batch_size().set(new_max_tx_batch_size);
    }

    #[only_owner]
    #[endpoint(setMaxTxBatchBlockDuration)]
    fn set_max_tx_batch_block_duration(&self, new_max_tx_batch_block_duration: u64) {
        require!(
            new_max_tx_batch_block_duration > 0,
            "Max tx batch block duration must be more than 0"
        );

        self.max_tx_batch_block_duration()
            .set(new_max_tx_batch_block_duration);
    }

    // views

    #[view(getCurrentTxBatch)]
    fn get_current_tx_batch(&self) -> OptionalValue<TxBatchSplitInFields<Self::Api>> {
        let first_batch_id = self.first_batch_id().get();
        let first_batch = self.pending_batches(first_batch_id);

        if self.is_batch_full(&first_batch, first_batch_id, first_batch_id)
            && self.is_batch_final(&first_batch)
        {
            let mut result_vec = MultiValueEncoded::new();
            for tx in first_batch.iter() {
                result_vec.push(tx.into_multiresult());
            }

            return OptionalValue::Some((first_batch_id, result_vec).into());
        }

        OptionalValue::None
    }

    #[view(getFirstBatchAnyStatus)]
    fn get_first_batch_any_status(&self) -> OptionalValue<TxBatchSplitInFields<Self::Api>> {
        let first_batch_id = self.first_batch_id().get();
        self.get_batch(first_batch_id)
    }

    #[view(getBatch)]
    fn get_batch(&self, batch_id: BatchId) -> OptionalValue<TxBatchSplitInFields<Self::Api>> {
        let tx_batch = self.pending_batches(batch_id);
        if tx_batch.is_empty() {
            return OptionalValue::None;
        }

        let mut result_vec = MultiValueEncoded::new();
        for tx in tx_batch.iter() {
            result_vec.push(tx.into_multiresult());
        }

        OptionalValue::Some((batch_id, result_vec).into())
    }

    #[view(getBatchStatus)]
    fn get_batch_status(&self, batch_id: BatchId) -> BatchStatus<Self::Api> {
        let first_batch_id = self.first_batch_id().get();
        if batch_id < first_batch_id {
            return BatchStatus::AlreadyProcessed;
        }

        let tx_batch = self.pending_batches(batch_id);
        if tx_batch.is_empty() {
            return BatchStatus::Empty;
        }

        if self.is_batch_full(&tx_batch, batch_id, first_batch_id) {
            if batch_id == first_batch_id {
                return BatchStatus::WaitingForSignatures;
            } else {
                return BatchStatus::Full;
            }
        }

        let mut tx_ids = ManagedVec::new();
        for tx in tx_batch.iter() {
            tx_ids.push(tx.nonce);
        }

        let max_tx_batch_block_duration = self.max_tx_batch_block_duration().get();
        let first_tx_in_batch_block_nonce = tx_batch.get_first_tx().block_nonce;

        BatchStatus::PartiallyFull {
            end_block_nonce: first_tx_in_batch_block_nonce + max_tx_batch_block_duration,
            tx_ids,
        }
    }

    // private

    fn add_to_batch(&self, transaction: Transaction<Self::Api>) -> BatchId {
        let default_gas_limit = self.sovereign_tx_gas_limit().get();
        let first_batch_id = self.first_batch_id().get();
        let last_batch_id = self.last_batch_id().get();
        let mut last_batch = self.pending_batches(last_batch_id);

        let gas_cost = match &transaction.opt_transfer_data {
            Some(transfer_data) => transfer_data.gas_limit,
            None => transaction.tokens.len() as u64 * default_gas_limit,
        };

        let gas_cost_mapper = self.total_gas_cost(last_batch_id);
        let last_batch_total_gas_cost = gas_cost_mapper.get();
        let new_total_gas_cost = last_batch_total_gas_cost + gas_cost;

        if self.is_batch_full(&last_batch, last_batch_id, first_batch_id)
            || new_total_gas_cost > MAX_GAS_LIMIT_PER_BATCH
        {
            let (new_batch_id, _) = self.create_new_batch(transaction);
            self.total_gas_cost(new_batch_id).set(gas_cost);

            new_batch_id
        } else {
            last_batch.push(transaction);
            gas_cost_mapper.set(new_total_gas_cost);

            last_batch_id
        }
    }

    // optimized to prevent reading/storing the batch over and over
    fn add_multiple_tx_to_batch(
        &self,
        transactions: &ManagedVec<Transaction<Self::Api>>,
    ) -> ManagedVec<BatchId> {
        if transactions.is_empty() {
            return ManagedVec::new();
        }

        let default_gas_limit = self.sovereign_tx_gas_limit().get();
        let first_batch_id = self.first_batch_id().get();
        let mut last_batch_id = self.last_batch_id().get();
        let mut last_batch = self.pending_batches(last_batch_id);
        let mut batch_ids = ManagedVec::new();
        let mut total_gas_cost = self.total_gas_cost(last_batch_id).get();

        for tx in transactions {
            let gas_cost = match &tx.opt_transfer_data {
                Some(transfer_data) => transfer_data.gas_limit,
                None => tx.tokens.len() as u64 * default_gas_limit,
            };

            let new_total_gas_cost = total_gas_cost + gas_cost;
            if self.is_batch_full(&last_batch, last_batch_id, first_batch_id)
                || new_total_gas_cost > MAX_GAS_LIMIT_PER_BATCH
            {
                self.total_gas_cost(last_batch_id).set(total_gas_cost);
                total_gas_cost = gas_cost;

                (last_batch_id, last_batch) = self.create_new_batch(tx);
            } else {
                total_gas_cost = new_total_gas_cost;
                last_batch.push(tx);
            }

            batch_ids.push(last_batch_id);
        }

        self.total_gas_cost(last_batch_id).set(total_gas_cost);

        batch_ids
    }

    fn create_new_batch(
        &self,
        transaction: Transaction<Self::Api>,
    ) -> (BatchId, TxBatchMapper<Self::Api>) {
        let last_batch_id = self.last_batch_id().get();
        let new_batch_id = last_batch_id + 1;

        let mut new_batch = self.pending_batches(new_batch_id);
        new_batch.push(transaction);

        self.last_batch_id().set(new_batch_id);

        (new_batch_id, new_batch)
    }

    fn is_batch_full(
        &self,
        tx_batch: &TxBatchMapper<Self::Api>,
        batch_id: BatchId,
        first_batch_id: BatchId,
    ) -> bool {
        if tx_batch.is_empty() {
            return false;
        }

        let max_batch_size = self.max_tx_batch_size().get();
        if tx_batch.len() == max_batch_size {
            return true;
        }

        // if this is not the first batch, we ignore the timestamp checks
        // we only check for max len
        if batch_id > first_batch_id {
            return false;
        }

        let current_block_nonce = self.blockchain().get_block_nonce();
        let first_tx_in_batch_block_nonce = tx_batch.get_first_tx().block_nonce;

        // reorg protection
        if current_block_nonce < first_tx_in_batch_block_nonce {
            return false;
        }

        let block_diff = current_block_nonce - first_tx_in_batch_block_nonce;
        let max_tx_batch_block_duration = self.max_tx_batch_block_duration().get();

        block_diff >= max_tx_batch_block_duration
    }

    fn is_batch_final(&self, tx_batch: &TxBatchMapper<Self::Api>) -> bool {
        if tx_batch.is_empty() {
            return false;
        }

        let last_tx_in_batch = tx_batch.get_last_tx();
        let current_block = self.blockchain().get_block_nonce();

        // reorg protection
        if current_block < last_tx_in_batch.block_nonce {
            return false;
        }

        let block_diff = current_block - last_tx_in_batch.block_nonce;

        block_diff > MIN_BLOCKS_FOR_FINALITY
    }

    fn clear_first_batch(&self, mapper: &mut TxBatchMapper<Self::Api>) {
        let first_batch_id = self.first_batch_id().get();
        let new_first_batch_id = first_batch_id + 1;

        // for the case when the last existing batch was processed
        // otherwise, we'd create a batch with the same ID again
        self.last_batch_id().update(|last_batch_id| {
            if *last_batch_id == first_batch_id {
                *last_batch_id = new_first_batch_id;
            }
        });
        self.first_batch_id().set(new_first_batch_id);

        mapper.clear();
    }

    fn get_and_save_next_tx_id(&self) -> TxNonce {
        self.last_tx_nonce().update(|last_tx_nonce| {
            *last_tx_nonce += 1;
            *last_tx_nonce
        })
    }

    // storage

    #[view(getFirstBatchId)]
    #[storage_mapper("firstBatchId")]
    fn first_batch_id(&self) -> SingleValueMapper<BatchId>;

    #[view(getLastBatchId)]
    #[storage_mapper("lastBatchId")]
    fn last_batch_id(&self) -> SingleValueMapper<BatchId>;

    #[storage_mapper("pendingBatches")]
    fn pending_batches(&self, batch_id: BatchId) -> TxBatchMapper<Self::Api>;

    #[storage_mapper("totalGasCost")]
    fn total_gas_cost(&self, batch_id: BatchId) -> SingleValueMapper<GasLimit>;

    #[storage_mapper("lastTxNonce")]
    fn last_tx_nonce(&self) -> SingleValueMapper<TxNonce>;

    // configurable

    #[storage_mapper("maxTxBatchSize")]
    fn max_tx_batch_size(&self) -> SingleValueMapper<usize>;

    #[storage_mapper("maxTxBatchBlockDuration")]
    fn max_tx_batch_block_duration(&self) -> SingleValueMapper<u64>;

    #[view(getSovereignTxGasLimit)]
    #[storage_mapper("sovereignTxGasLimit")]
    fn sovereign_tx_gas_limit(&self) -> SingleValueMapper<GasLimit>;
}
