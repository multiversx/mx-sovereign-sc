use bls_signature::BlsSignature;

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait Multisigverifier:
{
    #[init]
    fn init(&self, bls_pub_keys: MultiValueEncoded<ManagedAddress>) {
        // self.bls_pub_keys().get_or_create_users(bls_pub_keys.into_iter(), |mut user_id, _| user_id += 1);
        // or
        for key in bls_pub_keys {
        self.pub_bls_keys().push(&key);
      }
   }
    
    #[endpoint(registerBridgeOps)]
    fn register_bridge_operations(
        &self,
        hash_of_hashes: ManagedBuffer,
        hash_of_bridge_ops: ManagedBuffer,
        signature: BlsSignature<Self::Api>,
        signature_data: &ManagedBuffer,
    ) {

        let caller = self.blockchain().get_caller();
        let is_bls_valid = self.verify_bls_signature(signature_data, &signature, caller);
        
        if is_bls_valid {
            self.operations_mapper().insert(hash_of_hashes.clone(), hash_of_bridge_ops);
        }
    }

    fn verify_bls_signature(
        &self,
        signature_data: &ManagedBuffer,
        signature: &BlsSignature<Self::Api>,
        user: ManagedAddress
    ) -> bool {
        let is_bls_valid = self.crypto().verify_bls(
            user.as_managed_buffer(), 
            signature_data, 
            signature.as_managed_buffer()
        );
        let signatures_count = self.signatures().get();
        let bls_pub_keys = self.bls_pub_keys().get_user_count() as u32;

        if is_bls_valid && signatures_count > 2/3 * bls_pub_keys {
            return true
        }

        false
    }

    #[storage_mapper("isValid")]
    fn is_valid(&self) -> SingleValueMapper<bool>;

    #[storage_mapper("bls_pub_keys")]
    fn pub_bls_keys(&self) -> VecMapper<ManagedAddress>;

    #[storage_mapper("board_members")]
    fn bls_pub_keys(&self) -> UserMapper; 

    #[storage_mapper("signers")]
    fn signatures(&self) -> SingleValueMapper<u32>;

    #[storage_mapper("operations_mapper")]
    fn operations_mapper(&self) -> MapMapper<ManagedBuffer, ManagedBuffer>;
}
