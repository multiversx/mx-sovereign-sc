multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use max_bridged_amount_module::ProxyTrait as _;
use multi_transfer_esdt::ProxyTrait as _;
use multiversx_sc_modules::pause::ProxyTrait as _;
use token_module::ProxyTrait as _;
use tx_batch_module::ProxyTrait as _;

#[multiversx_sc::module]
pub trait SetupModule:
    crate::multisig_general::MultisigGeneralModule
    + crate::storage::StorageModule
    + crate::util::UtilModule
    + multiversx_sc_modules::pause::PauseModule
{
    #[only_owner]
    #[endpoint(upgradeChildContractFromSource)]
    fn upgrade_child_contract_from_source(
        &self,
        child_sc_address: ManagedAddress,
        source_address: ManagedAddress,
        is_payable: bool,
        init_args: MultiValueEncoded<ManagedBuffer>,
    ) {
        let mut metadata = CodeMetadata::UPGRADEABLE;
        if is_payable {
            metadata |= CodeMetadata::PAYABLE_BY_SC;
        }

        let gas = self.blockchain().get_gas_left();
        self.send_raw().upgrade_from_source_contract(
            &child_sc_address,
            gas,
            &BigUint::zero(),
            &source_address,
            metadata,
            &init_args.to_arg_buffer(),
        );
    }

    #[only_owner]
    #[endpoint(addBoardMember)]
    fn add_board_member_endpoint(&self, board_member: ManagedAddress) {
        self.add_board_member(&board_member);
    }

    #[only_owner]
    #[endpoint(removeUser)]
    fn remove_user(&self, board_member: ManagedAddress) {
        self.remove_board_member(&board_member);
        let num_board_members = self.num_board_members().get();
        require!(num_board_members > 0, "cannot remove all board members");
        require!(
            self.quorum().get() <= num_board_members,
            "quorum cannot exceed board size"
        );
    }

    /// Cuts a fixed amount from a board member's stake.
    /// This should be used only in cases where the board member
    /// is being actively malicious.
    ///
    /// After stake is cut, the board member would have to stake again
    /// to be able to sign actions.
    #[only_owner]
    #[endpoint(slashBoardMember)]
    fn slash_board_member(&self, board_member: ManagedAddress) {
        self.remove_user(board_member.clone());

        let slash_amount = self.slash_amount().get();

        // remove slashed amount from user stake amountself
        self.amount_staked(&board_member)
            .update(|stake| *stake -= &slash_amount);

        // add it to total slashed amount pool
        self.slashed_tokens_amount()
            .update(|slashed_amt| *slashed_amt += slash_amount);
    }

    #[only_owner]
    #[endpoint(changeQuorum)]
    fn change_quorum(&self, new_quorum: usize) {
        require!(
            new_quorum <= self.num_board_members().get(),
            "quorum cannot exceed board size"
        );
        self.quorum().set(new_quorum);
    }

    /// Maps an ESDT token to a Sovereign ESDT token. Used by relayers.
    #[only_owner]
    #[endpoint(addMapping)]
    fn add_mapping(&self, sov_token_id: TokenIdentifier, elrond_token_id: TokenIdentifier) {
        require!(
            self.sov_token_id_for_elrond_token_id(&elrond_token_id)
                .is_empty(),
            "Mapping already exists for token ID"
        );
        require!(
            self.elrond_token_id_for_sov_token_id(&sov_token_id)
                .is_empty(),
            "Mapping already exists for ERC20 token"
        );

        self.sov_token_id_for_elrond_token_id(&elrond_token_id)
            .set(&sov_token_id);
        self.elrond_token_id_for_sov_token_id(&sov_token_id)
            .set(&elrond_token_id);
    }

    #[only_owner]
    #[endpoint(clearMapping)]
    fn clear_mapping(&self, sov_token_id: TokenIdentifier, elrond_token_id: TokenIdentifier) {
        require!(
            !self
                .sov_token_id_for_elrond_token_id(&elrond_token_id)
                .is_empty(),
            "Mapping does not exist for ERC20 token"
        );
        require!(
            !self
                .elrond_token_id_for_sov_token_id(&sov_token_id)
                .is_empty(),
            "Mapping does not exist for token id"
        );

        let mapped_sov_token_id = self
            .sov_token_id_for_elrond_token_id(&elrond_token_id)
            .get();
        let mapped_elrond_token_id = self.elrond_token_id_for_sov_token_id(&sov_token_id).get();

        require!(
            sov_token_id == mapped_sov_token_id && elrond_token_id == mapped_elrond_token_id,
            "Invalid mapping"
        );

        self.sov_token_id_for_elrond_token_id(&elrond_token_id)
            .clear();
        self.elrond_token_id_for_sov_token_id(&sov_token_id).clear();
    }

    #[only_owner]
    #[endpoint(pauseEsdtSafe)]
    fn pause_esdt_safe(&self) {
        let _: IgnoreValue = self
            .get_esdt_safe_proxy_instance()
            .pause_endpoint()
            .execute_on_dest_context();
    }

    #[only_owner]
    #[endpoint(unpauseEsdtSafe)]
    fn unpause_esdt_safe(&self) {
        let _: IgnoreValue = self
            .get_esdt_safe_proxy_instance()
            .unpause_endpoint()
            .execute_on_dest_context();
    }

    #[only_owner]
    #[endpoint(esdtSafeAddTokenToWhitelist)]
    fn esdt_safe_add_token_to_whitelist(&self, token_id: TokenIdentifier, ticker: ManagedBuffer) {
        let _: IgnoreValue = self
            .get_esdt_safe_proxy_instance()
            .add_token_to_whitelist(token_id, ticker)
            .execute_on_dest_context();
    }

    #[only_owner]
    #[endpoint(esdtSafeRemoveTokenFromWhitelist)]
    fn esdt_safe_remove_token_from_whitelist(&self, token_id: TokenIdentifier) {
        let _: IgnoreValue = self
            .get_esdt_safe_proxy_instance()
            .remove_token_from_whitelist(token_id)
            .execute_on_dest_context();
    }

    /// Sets maximum batch size for the EsdtSafe SC.
    /// If a batch reaches this amount of transactions, it is considered full,
    /// and a new incoming transaction will be put into a new batch.
    #[only_owner]
    #[endpoint(esdtSafeSetMaxTxBatchSize)]
    fn esdt_safe_set_max_tx_batch_size(&self, new_max_tx_batch_size: usize) {
        let _: IgnoreValue = self
            .get_esdt_safe_proxy_instance()
            .set_max_tx_batch_size(new_max_tx_batch_size)
            .execute_on_dest_context();
    }

    /// Sets the maximum block duration in which an EsdtSafe batch accepts transactions
    /// For a batch to be considered "full", it has to either reach `maxTxBatchSize` transactions,
    /// or have txBatchBlockDuration blocks pass since the first tx was added in the batch
    #[only_owner]
    #[endpoint(esdtSafeSetMaxTxBatchBlockDuration)]
    fn esdt_safe_set_max_tx_batch_block_duration(&self, new_max_tx_batch_block_duration: u64) {
        let _: IgnoreValue = self
            .get_esdt_safe_proxy_instance()
            .set_max_tx_batch_block_duration(new_max_tx_batch_block_duration)
            .execute_on_dest_context();
    }

    /// Sets the maximum bridged amount for the token for the Elrond -> Sovereign direction.
    /// Any attempt to transfer over this amount will be rejected.
    #[only_owner]
    #[endpoint(esdtSafeSetMaxBridgedAmountForToken)]
    fn esdt_safe_set_max_bridged_amount_for_token(
        &self,
        token_id: TokenIdentifier,
        max_amount: BigUint,
    ) {
        let _: IgnoreValue = self
            .get_esdt_safe_proxy_instance()
            .set_max_bridged_amount(token_id, max_amount)
            .execute_on_dest_context();
    }

    /// Same as the function above, but for Sovereign -> Elrond transactions.
    #[only_owner]
    #[endpoint(multiTransferEsdtSetMaxBridgedAmountForToken)]
    fn multi_transfer_esdt_set_max_bridged_amount_for_token(
        &self,
        token_id: TokenIdentifier,
        max_amount: BigUint,
    ) {
        let _: IgnoreValue = self
            .get_multi_transfer_esdt_proxy_instance()
            .set_max_bridged_amount(token_id, max_amount)
            .execute_on_dest_context();
    }

    /// Any failed Sovereign -> Elrond transactions are added into so-called "refund batches"
    /// This configures the size of a batch.
    #[only_owner]
    #[endpoint(multiTransferEsdtSetMaxRefundTxBatchSize)]
    fn multi_transfer_esdt_set_max_refund_tx_batch_size(&self, new_max_tx_batch_size: usize) {
        let _: IgnoreValue = self
            .get_multi_transfer_esdt_proxy_instance()
            .set_max_tx_batch_size(new_max_tx_batch_size)
            .execute_on_dest_context();
    }

    /// Max block duration for refund batches. Default is "infinite" (u64::MAX)
    /// and only max batch size matters
    #[only_owner]
    #[endpoint(multiTransferEsdtSetMaxRefundTxBatchBlockDuration)]
    fn multi_transfer_esdt_set_max_refund_tx_batch_block_duration(
        &self,
        new_max_tx_batch_block_duration: u64,
    ) {
        let _: IgnoreValue = self
            .get_multi_transfer_esdt_proxy_instance()
            .set_max_tx_batch_block_duration(new_max_tx_batch_block_duration)
            .execute_on_dest_context();
    }

    /// Sets the wrapping contract address.
    /// This contract is used to map multiple tokens to a universal one.
    /// Useful in cases where a single token (USDC for example)
    /// is being transferred from multiple chains.
    ///
    /// They will all have different token IDs, but can be swapped 1:1 in the wrapping SC.
    /// The wrapping is done automatically, so the user only receives the universal token.
    #[only_owner]
    #[endpoint(multiTransferEsdtSetWrappingContractAddress)]
    fn multi_transfer_esdt_set_wrapping_contract_address(
        &self,
        opt_wrapping_contract_address: OptionalValue<ManagedAddress>,
    ) {
        let _: IgnoreValue = self
            .get_multi_transfer_esdt_proxy_instance()
            .set_wrapping_contract_address(opt_wrapping_contract_address)
            .execute_on_dest_context();
    }
}
