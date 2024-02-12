use bls_signature::{self, BlsSignature};
use esdt_safe::from_sovereign::{self, transfer_tokens};
use transaction::TransferData;

use crate::utils::{self, UtilsModule};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait BridgeOperationsModule
{
    fn register_bridge_operations(
        &self,
        hash_of_hashes: ManagedBuffer,
        hash_of_bridge_ops: ManagedBuffer,
        signature: BlsSignature<Self::Api>,
        signature_data: &ManagedBuffer,
        //use Transaction nonce, token_data
    ) {

        let caller = self.blockchain().get_caller();
        let is_batch_valid = self.check_validity(signature_data, &signature, caller);
        self.is_operations_batch_valid().insert(hash_of_hashes.clone(), is_batch_valid);
        self.operations_mapper().insert(hash_of_hashes.clone(), hash_of_bridge_ops);
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

        // assert!(
        //     !signers.get(caller),
        //     "Caller already signed the operations!"
        // );

        // self.signatures() = self.signatures() + 1; 
    }

    fn check_validity(
        &self,
        signature_data: &ManagedBuffer,
        signature: &BlsSignature<Self::Api>,
        user: ManagedAddress
    ) -> bool {
        // let is_bls_valid = self.crypto().verify_bls(user.as_managed_buffer(), signature_data, signature);
        // let signature_count = self.signatures().len();

        // if is_bls_valid && signature_count > 2/3 * self.board_members() {
        //     return true
        // }

        false
    }
    
    #[endpoint(execBridgeOp)]
    fn execute_bridge_operation_endpoint(
        &self,
        operation_hash: ManagedBuffer,
        payments_list: MultiValueEncoded<EsdtTokenPayment<Self::Api>>,
        function_to_call: ManagedBuffer,
        args: MultiValueEncoded<ManagedBuffer>,
        gas_limit: u32
    ) {
        // require action not executed
        let caller = self.blockchain().get_caller();
        // require validator

        // self.execute_operation(operation_hash);

        // transfer_tokens::batch_transfer_esdt_token();
    }
    
    fn execute_operation(&self, operation_hash: ManagedBuffer, data: TransferData<Self::Api>) {
        let esdt_address = self.esdt_proxy_address().get();
        utils::contract_obj().get_esdt_safe_proxy_instance(esdt_address);
    }

    #[storage_mapper("esdt_proxy_address")]
    fn esdt_proxy_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("isValid")]
    fn is_valid(&self) -> SingleValueMapper<bool>;

    // maybe use a dictionary?
    #[storage_mapper("valid_batches")]
    fn is_operations_batch_valid(&self) -> MapMapper<ManagedBuffer, bool>;

    #[storage_mapper("board_members")]
    fn board_members(&self) -> UserMapper; 

    #[storage_mapper("signers")]
    fn signatures(&self) -> SingleValueMapper<u32>;

    #[storage_mapper("operations_mapper")]
    fn operations_mapper(&self) -> MapMapper<ManagedBuffer, ManagedBuffer>;
}
