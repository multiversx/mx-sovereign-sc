#![no_std]
#![allow(clippy::too_many_arguments)]

mod action;
mod multisig_general;
mod queries;
mod setup;
mod storage;
mod user_role;
mod util;

use action::Action;
use transaction::transaction_status::TransactionStatus;
use transaction::TxBatchSplitInFields;
use transaction::*;
use user_role::UserRole;

use esdt_safe::ProxyTrait as _;
use multi_transfer_esdt::ProxyTrait as _;
use tx_batch_module::ProxyTrait as _;

multiversx_sc::imports!();

/// Multi-signature smart contract implementation.
/// Acts like a wallet that needs multiple signers for any action performed.
#[multiversx_sc::contract]
pub trait Multisig:
    multisig_general::MultisigGeneralModule
    + setup::SetupModule
    + storage::StorageModule
    + util::UtilModule
    + queries::QueriesModule
    + multiversx_sc_modules::pause::PauseModule
{
    /// EsdtSafe and MultiTransferEsdt are expected to be deployed and configured separately,
    /// and then having their ownership changed to this Multisig SC.
    #[init]
    fn init(
        &self,
        esdt_safe_sc_address: ManagedAddress,
        multi_transfer_sc_address: ManagedAddress,
        required_stake: BigUint,
        slash_amount: BigUint,
        quorum: usize,
        board: MultiValueEncoded<ManagedAddress>,
    ) {
        let mut duplicates = false;
        let board_len = board.len();
        self.user_mapper()
            .get_or_create_users(board.into_iter(), |user_id, new_user| {
                if !new_user {
                    duplicates = true;
                }
                self.user_id_to_role(user_id).set(UserRole::BoardMember);
            });
        require!(!duplicates, "duplicate board member");

        self.num_board_members()
            .update(|nr_board_members| *nr_board_members += board_len);
        self.change_quorum(quorum);

        require!(
            slash_amount <= required_stake,
            "slash amount must be less than or equal to required stake"
        );
        self.required_stake_amount().set(&required_stake);
        self.slash_amount().set(&slash_amount);

        require!(
            self.blockchain().is_smart_contract(&esdt_safe_sc_address),
            "Esdt Safe address is not a Smart Contract address"
        );
        self.esdt_safe_address().set(&esdt_safe_sc_address);

        require!(
            self.blockchain()
                .is_smart_contract(&multi_transfer_sc_address),
            "Multi Transfer address is not a Smart Contract address"
        );
        self.multi_transfer_esdt_address()
            .set(&multi_transfer_sc_address);

        self.set_paused(true);
    }

    #[endpoint]
    fn upgrade(&self) {}

    /// Board members have to stake a certain amount of EGLD
    /// before being allowed to sign actions
    #[payable("EGLD")]
    #[endpoint]
    fn stake(&self) {
        let egld_payment = self.call_value().egld_value().clone_value();
        let caller = self.blockchain().get_caller();
        let caller_role = self.user_role(&caller);
        require!(
            caller_role == UserRole::BoardMember,
            "Only board members can stake"
        );

        self.amount_staked(&caller)
            .update(|amount_staked| *amount_staked += egld_payment);
    }

    #[endpoint]
    fn unstake(&self, amount: BigUint) {
        let caller = self.blockchain().get_caller();
        let amount_staked = self.amount_staked(&caller).get();
        require!(
            amount <= amount_staked,
            "can't unstake more than amount staked"
        );

        let remaining_stake = &amount_staked - &amount;
        if self.user_role(&caller) == UserRole::BoardMember {
            let required_stake_amount = self.required_stake_amount().get();
            require!(
                remaining_stake >= required_stake_amount,
                "can't unstake, must keep minimum amount as insurance"
            );
        }

        self.amount_staked(&caller).set(&remaining_stake);
        self.send().direct_egld(&caller, &amount);
    }

    // ESDT Safe SC calls

    /// After a batch is processed on the Sovereign side,
    /// the EsdtSafe expects a list of statuses of said transactions (success or failure).
    ///
    /// This endpoint proposes an action to set the statuses to a certain list of values.
    /// Nothing is changed in the EsdtSafe contract until the action is signed and executed.
    #[endpoint(proposeEsdtSafeSetCurrentTransactionBatchStatus)]
    fn propose_esdt_safe_set_current_transaction_batch_status(
        &self,
        esdt_safe_batch_id: u64,
        tx_batch_status: MultiValueEncoded<TransactionStatus>,
    ) -> usize {
        let call_result: OptionalValue<TxBatchSplitInFields<Self::Api>> = self
            .get_esdt_safe_proxy_instance()
            .get_current_tx_batch()
            .execute_on_dest_context();
        let (current_batch_id, current_batch_transactions) = match call_result {
            OptionalValue::Some(batch) => batch.into_tuple(),
            OptionalValue::None => sc_panic!("Current batch is empty"),
        };
        let statuses_vec = tx_batch_status.to_vec();

        require!(
            self.action_id_for_set_current_transaction_batch_status(esdt_safe_batch_id)
                .get(&statuses_vec)
                .is_none(),
            "Action already proposed"
        );

        let current_batch_len = current_batch_transactions.raw_len() / TX_MULTIRESULT_NR_FIELDS;
        let status_batch_len = statuses_vec.len();
        require!(
            current_batch_len == status_batch_len,
            "Number of statuses provided must be equal to number of transactions in current batch"
        );
        require!(
            esdt_safe_batch_id == current_batch_id,
            "Current EsdtSafe tx batch does not have the provided ID"
        );

        let action_id = self.propose_action(Action::SetCurrentTransactionBatchStatus {
            esdt_safe_batch_id,
            tx_batch_status: statuses_vec.clone(),
        });

        self.action_id_for_set_current_transaction_batch_status(esdt_safe_batch_id)
            .insert(statuses_vec, action_id);

        action_id
    }

    // Multi-transfer ESDT SC calls

    /// Proposes a batch of Sovereign -> Elrond transfers.
    /// Transactions have to be separated by fields, in the following order:
    /// Sender Address, Destination Address, Token ID, Amount, Tx Nonce
    #[endpoint(proposeMultiTransferEsdtBatch)]
    fn propose_multi_transfer_esdt_batch(
        &self,
        sov_batch_id: u64,
        transfers: MultiValueEncoded<TxAsMultiValue<Self::Api>>,
    ) -> usize {
        let next_sov_batch_id = self.last_executed_sov_batch_id().get() + 1;
        require!(
            sov_batch_id == next_sov_batch_id,
            "Can only propose for next batch ID"
        );

        let transfers_as_sov_tx = self.transfers_multi_value_to_sov_tx_vec(transfers);
        self.require_valid_sov_tx_ids(&transfers_as_sov_tx);

        let batch_hash = self.hash_sov_tx_batch(&transfers_as_sov_tx);
        require!(
            self.batch_id_to_action_id_mapping(sov_batch_id)
                .get(&batch_hash)
                .is_none(),
            "This batch was already proposed"
        );

        let action_id = self.propose_action(Action::BatchTransferEsdtToken {
            sov_batch_id,
            transfers: transfers_as_sov_tx,
        });

        self.batch_id_to_action_id_mapping(sov_batch_id)
            .insert(batch_hash, action_id);

        action_id
    }

    /// Failed Sovereign -> Elrond transactions are saved in the MultiTransfer SC
    /// as "refund transactions", and stored in batches, using the same mechanism as EsdtSafe.
    ///
    /// This function moves the first refund batch into the EsdtSafe SC,
    /// converting the transactions into Elrond -> Sovereign transactions
    /// and adding them into EsdtSafe batches
    #[only_owner]
    #[endpoint(moveRefundBatchToSafe)]
    fn move_refund_batch_to_safe(&self) {
        let opt_refund_batch_fields: OptionalValue<TxBatchSplitInFields<Self::Api>> = self
            .get_multi_transfer_esdt_proxy_instance()
            .get_and_clear_first_refund_batch()
            .execute_on_dest_context();

        if let OptionalValue::Some(refund_batch_fields) = opt_refund_batch_fields {
            let (_batch_id, all_tx_fields) = refund_batch_fields.into_tuple();
            let mut refund_batch = ManagedVec::new();

            for tx_fields in all_tx_fields {
                refund_batch.push(Transaction::from(tx_fields));
            }

            let _: IgnoreValue = self
                .get_esdt_safe_proxy_instance()
                .add_refund_batch(refund_batch)
                .execute_on_dest_context();
        }
    }

    /// Proposers and board members use this to launch signed actions.
    #[endpoint(performAction)]
    fn perform_action_endpoint(&self, action_id: usize) {
        require!(
            !self.action_mapper().item_is_empty(action_id),
            "Action was already executed"
        );

        let caller_address = self.blockchain().get_caller();
        let caller_role = self.get_user_role(&caller_address);
        require!(
            caller_role.is_board_member(),
            "only board members can perform actions"
        );
        require!(
            self.quorum_reached(action_id),
            "quorum has not been reached"
        );
        require!(self.not_paused(), "No actions may be executed while paused");

        self.perform_action(action_id);
    }

    // private

    fn perform_action(&self, action_id: usize) {
        let action = self.action_mapper().get(action_id);
        self.clear_action(action_id);

        match action {
            Action::Nothing => {}
            Action::SetCurrentTransactionBatchStatus {
                esdt_safe_batch_id,
                tx_batch_status,
            } => {
                let mut action_ids_mapper =
                    self.action_id_for_set_current_transaction_batch_status(esdt_safe_batch_id);

                // if there's only one proposed action,
                // the action was already cleared at the beginning of this function
                if action_ids_mapper.len() > 1 {
                    for act_id in action_ids_mapper.values() {
                        self.clear_action(act_id);
                    }
                }

                action_ids_mapper.clear();

                let _: IgnoreValue = self
                    .get_esdt_safe_proxy_instance()
                    .set_transaction_batch_status(
                        esdt_safe_batch_id,
                        MultiValueEncoded::from(tx_batch_status),
                    )
                    .execute_on_dest_context();
            }
            Action::BatchTransferEsdtToken {
                sov_batch_id,
                transfers,
            } => {
                let mut action_ids_mapper = self.batch_id_to_action_id_mapping(sov_batch_id);

                // if there's only one proposed action,
                // the action was already cleared at the beginning of this function
                if action_ids_mapper.len() > 1 {
                    for act_id in action_ids_mapper.values() {
                        self.clear_action(act_id);
                    }
                }

                action_ids_mapper.clear();
                self.last_executed_sov_batch_id().update(|id| *id += 1);

                let last_tx_index = transfers.len() - 1;
                let last_tx = transfers.get(last_tx_index);
                self.last_executed_sov_tx_id().set(last_tx.nonce);

                let transfers_multi: MultiValueEncoded<Self::Api, Transaction<Self::Api>> =
                    transfers.into();
                let _: IgnoreValue = self
                    .get_multi_transfer_esdt_proxy_instance()
                    .batch_transfer_esdt_token(sov_batch_id, transfers_multi)
                    .execute_on_dest_context();
            }
        }
    }
}
