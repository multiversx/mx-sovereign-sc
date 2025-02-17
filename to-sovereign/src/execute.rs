use operation::Operation;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait ExecuteModule {
    #[endpoint(execute)]
    fn execute(&self, hash_of_hashes: ManagedBuffer, operation: Operation<Self::Api>) {
        let is_sovereign_chain = self.is_sovereign_chain().get();
        require!(
            !is_sovereign_chain,
            "Invalid method to call in current chain"
        );

        require!(self.not_paused(), "Cannot transfer while paused");

        let operation_hash = self.calculate_operation_hash(&operation);

        self.lock_operation_hash(&operation_hash, &hash_of_hashes);

        let minted_operation_tokens = self.mint_tokens(&operation.tokens);
        let operation_tuple = OperationTuple {
            op_hash: operation_hash,
            operation,
        };

        self.distribute_payments(&hash_of_hashes, &operation_tuple, &minted_operation_tokens);
    }
}
