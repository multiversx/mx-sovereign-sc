use bls_signature::BlsSignature;

pub struct PendingOperations<M: ManagedTypeApi> {
    hash_of_hashes: ManagedBuffer<M>,
    bridge_operations_hash: MultiValueEncoded<M, ManagedBuffer<M>>,
    bls_signature: BlsSignature<M>
}

pub struct Operation<M: ManagedTypeApi> {
    address: ManagedAddress<M>,
    token_info: MultiValue3<TokenIdentifier<M>, ManagedBuffer<M>, ManagedBuffer<M>>,
    function_to_call: ManagedBuffer<M>,
    args: ManagedBuffer<M>,
    gas_limit: BigUint<M>
} 
multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait BridgeOperationsModule: 
    bls_signature::BlsSignatureModule
{
    fn register_bridge_operations(
        &self,
        operations: PendingOperations<Self::Api>
    ) {
        let caller_address = self.blockchain().get_caller();

        let current_operations = self.operations_mapper().get();
        require!(
            !current_operations.contains(&operations.hash_of_hashes),
            "Hash of operations already exists" 
        );

        current_operations.push(operations);
    }

    fn sign(
        &self, 
        operations: PendingOperations<Self::Api>,
        signature_data: &ManagedBuffer,
        signature: &BlsSignature<Self::Api>
    ) {
        let caller = self.blockchain().get_caller();
        let signers = self.signatures().get();

        assert!(
            !signers.contains(caller),
            "Caller already signed the operations!"
        );

        let is_valid = self.crypto().verify_bls(caller.as_managed_buffer(), signature_data, signature);

        if is_valid {
             self.signatures().push(caller); 
        }
    }
    
    fn execute_bridge_operations(
        &self,
        operations: PendingOperations<Self::Api>,
        operation_hash: ManagedBuffer,

    ) {
        
    }

    #[view(signatures)]
    #[storage_mapper("signature")]
    fn signatures(&self) -> MultiValueEncoded<ManagedAddress>;

    // maybe use a dictionary?
    #[view(getOperations)]
    #[storage_mapper("operations")]
    fn pending_operations_mapper(&self) -> MultiValueEncoded<PendingOperations<Self::Api>>;
}
