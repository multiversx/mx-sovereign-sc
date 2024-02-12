use bls_signature::{self, BlsSignature};
use esdt_safe::from_sovereign::{self, transfer_tokens};
use transaction::TransferData;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait BridgeOperationsModule:
    bls_signature::BlsSignatureModule
    // + from_sovereign::transfer_tokens::TransferTokensModule
    // + multiversx_sc_modules::pause::EndpointWrappers
    // + multiversx_sc_modules::pause::PauseModule
    // + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    fn register_bridge_operations(
        &self,
        hash_of_hashes: ManagedBuffer,
        hash_of_bridge_ops: ManagedVec<ManagedBuffer>,
        signature: BlsSignature<Self::Api>,
        signature_data: &ManagedBuffer,
        //use Transaction nonce, token_data
    ) {
        let caller = self.blockchain().get_caller();
        let is_batch_valid = self.check_validity(signature_data, signature, caller);
        self.is_operations_batch_valid().insert(hash_of_hashes, is_batch_valid);
        self.operations_mapper().insert(hash_of_hashes, hash_of_bridge_ops);
    }

    fn add_board_member(
        &self,
        user_address: &ManagedAddress,       
    ) {
        let user_id = self.board_members().get_or_create_user(user_address);
    }

    #[endpoint(sign)]
    fn sign(
        &self, 
        signature_data: &ManagedBuffer,
        signature: &BlsSignature<Self::Api>
    ) {
        let caller = self.blockchain().get_caller();
        let signers = self.all_signers().get();

        assert!(
            !signers.get(caller),
            "Caller already signed the operations!"
        );

        self.signatures().push(&caller); 
    }

    fn check_validity(
        &self,
        signature_data: &ManagedBuffer,
        signature: &BlsSignature<Self::Api>,
        user: ManagedAddress
    ) -> bool {
        let is_bls_valid = self.crypto().verify_bls(user.as_managed_buffer(), signature_data, signature);
        let signature_count = self.signatures().len();

        if is_bls_valid && signature_count > 2/3 * self.board_members() {
            true
        }

        false
    }
    
    #[endpoint(execBridgeOp)]
    fn execute_bridge_operation_endpoint(
        &self,
        operation_hash: ManagedBuffer,
    ) {
        // require action not executed
        let caller = self.blockchain().get_caller();
        // require validator

        self.execute_operation(operation_hash);

        // transfer_tokens::batch_transfer_esdt_token();
    }
    
    fn execute_operation(&self, operation_hash: ManagedBuffer, data: TransferData<Self::Api>) {
    }

    #[view(signatures)]
    #[storage_mapper("signature")]
    fn signatures(&self) -> VecMapper<ManagedAddress>;

    #[storage_mapper("isValid")]
    fn is_valid(&self) -> SingleValueMapper<bool>;

    // maybe use a dictionary?
    #[storage_mapper("valid_batches")]
    fn is_operations_batch_valid(&self) -> MapMapper<ManagedBuffer, bool>;

    #[storage_mapper("boardMembers")]
    fn board_members(&self) -> UserMapper; 

    #[storage_mapper("operations_mapper")]
    fn operations_mapper(&self) -> MapMapper<ManagedBuffer, ManagedBuffer>;
}
