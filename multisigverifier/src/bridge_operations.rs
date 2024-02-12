use bls_signature::{self, BlsSignature};
use esdt_safe::from_sovereign::{self, transfer_tokens::{self, ProxyTrait as _ }};
use transaction::TransferData;

use crate::utils::{self, UtilsModule};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait BridgeOperationsModule
{
    #[endpoint(registerBridgeOps)]
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
        self.operations_signatures_mapper().insert(hash_of_hashes.clone(), signature);
    }

    #[only_owner]
    #[endpoint(execBridgeOp)]
    fn execute_bridge_operation_endpoint(
        &self,
        operation_hash: ManagedBuffer,
        payments_list: MultiValueEncoded<EsdtTokenPayment<Self::Api>>,
        transfer_data: TransferData<Self::Api>
    ) {
        // require action not executed
        let caller = self.blockchain().get_caller();
        let caller_id = self.bls_pub_keys().get_user_id(&caller);
        let operation_validity = self.is_operations_batch_valid().get(&operation_hash).unwrap();

        require!(
            operation_validity,
            "Operation was not signed"
        );

        for payment in payments_list {
            self.execute_operation(&operation_hash, &payment, &transfer_data); 
        }

        // transfer_tokens::batch_transfer_esdt_token();
    }

    fn check_validity(
        &self,
        signature_data: &ManagedBuffer,
        signature: &BlsSignature<Self::Api>,
        user: ManagedAddress
    ) -> bool {
        let is_bls_valid = self.crypto().verify_bls(user.as_managed_buffer(), signature_data, signature.as_managed_buffer());
        let signatures_count = self.signatures().get();
        let bls_pub_keys = self.bls_pub_keys().get_user_count() as u32;

        if is_bls_valid && signatures_count > 2/3 * bls_pub_keys {
            return true
        }

        false
    }
   
    fn execute_operation(&self, operation_hash: &ManagedBuffer, payment: &EsdtTokenPayment, data: &TransferData<Self::Api>) {
        let esdt_address = self.esdt_proxy_address().get();
        let esdt_proxy = utils::contract_obj().get_esdt_safe_proxy_instance(esdt_address);
        let operation_signature = self.operations_signatures_mapper().get(operation_hash);
        // esdt_proxy.batch_transfer_esdt_token(operation_hash, operation_signature, payment);
    }

    #[storage_mapper("esdt_proxy_address")]
    fn esdt_proxy_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("isValid")]
    fn is_valid(&self) -> SingleValueMapper<bool>;

    // maybe use a dictionary?
    #[storage_mapper("valid_batches")]
    fn is_operations_batch_valid(&self) -> MapMapper<ManagedBuffer, bool>;

    #[storage_mapper("board_members")]
    fn bls_pub_keys(&self) -> UserMapper; 

    #[storage_mapper("signers")]
    fn signatures(&self) -> SingleValueMapper<u32>;

    #[storage_mapper("operations_mapper")]
    fn operations_mapper(&self) -> MapMapper<ManagedBuffer, ManagedBuffer>;

    #[storage_mapper("operations_signature")]
    fn operations_signatures_mapper(&self) -> MapMapper<ManagedBuffer, BlsSignature<Self::Api>>;
}
