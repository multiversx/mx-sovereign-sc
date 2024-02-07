use bls_signature::{self, BlsSignature};

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
pub trait BridgeOperationsModule 
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

    fn add_board_member(
        &self,
        user_address: &ManagedAddress,       
    ) {
        let user_id = self.board_members().get_or_create_user(user_address);
    }

    fn sign(
        &self, 
        operations: PendingOperations<Self::Api>,   //OutGointTxData
        signature_data: &ManagedBuffer,
        signature: &BlsSignature<Self::Api>
    ) {
        let caller = self.blockchain().get_caller();
        let signers = self.all_signers().get();

        assert!(
            !signers.contains(caller),
            "Caller already signed the operations!"
        );

        self.signatures().push(&caller); 
    }

    fn check_validity(
        &self,
        signature_data: &ManagedBuffer,
        signature: &BlsSignature<Self::Api>,
        user: ManagedAddress
    ) {
        let is_bls_valid = self.crypto().verify_bls(user.as_managed_buffer(), signature_data, signature);
        let signature_count = self.signatures().len();

        if is_bls_valid && signature_count > 2/3 * self.board_members() {
            self.is_valid().set(true);
        }

        self.is_valid().set(false);
    }
    
    #[endpoint(execBridgeOp)]
    fn execute_bridge_operation_endpoint(
        &self,
        operations: MultiValueEncoded<Self::Api, Operation<Self::Api>>,
        operation_hash: ManagedBuffer,
    ) {
        // require action not executed
        let caller = self.blockchain().get_caller();
        // require validator

        self.execute_operation(operation_hash);
    }
    
    fn execute_operation(&self, operation_hash: ManagedBuffer) {
        // remove operation from mapper
        let is_valid = self.is_valid().get();
        let operation = self.pending_operations_mapper().get(operation_hash);

        require!(
            is_valid,
            "The requirements for executing have not been met"
        );

        // exec operation.function to call with operation.args
    }

    #[view(signatures)]
    #[storage_mapper("signature")]
    fn signatures(&self) -> VecMapper<ManagedAddress>;

    #[storage_mapper("isValid")]
    fn is_valid(&self) -> SingleValueMapper<bool>;

    // maybe use a dictionary?
    #[view(getOperations)]
    #[storage_mapper("operations")]
    fn pending_operations_mapper(&self) -> MultiValueEncoded<Self::Api, PendingOperations<Self::Api>>;

    #[storage_mapper("boardMembers")]
    fn board_members(&self) -> UserMapper; 
}
