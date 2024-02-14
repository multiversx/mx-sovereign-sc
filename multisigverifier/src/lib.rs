use bls_signature::BlsSignature;

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait Multisigverifier:
  bls_signature::BlsSignatureModule
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
        hash_of_bridge_ops: MultiValueEncoded<ManagedBuffer>,
        signature: BlsSignature<Self::Api>,
    ) {
        let caller = self.blockchain().get_caller();
        let is_bls_valid = self.verify_bls(hash_of_bridge_ops.clone(), &signature, caller);
        
        if is_bls_valid { 
            for hash in hash_of_bridge_ops {
              self.pending_operations_mapper().insert(hash);
            }
        }
    }

    fn verify_bls (
        &self,
        transactions: MultiValueEncoded<ManagedBuffer>,
        signature: &BlsSignature<Self::Api>,
        user: ManagedAddress
    ) -> bool {
        let mut serialized_signature_data = ManagedBuffer::new();
        for transaction in transactions.into_iter() {
          let _ = transaction.dep_encode(&mut serialized_signature_data);
        }

        let is_bls_valid = self.crypto().verify_bls(
            user.as_managed_buffer(), 
            &serialized_signature_data, 
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
    fn pending_operations_mapper(&self) -> SetMapper<ManagedBuffer>; 
}
