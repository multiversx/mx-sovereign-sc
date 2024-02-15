use bls_signature::BlsSignature;
use transaction::TransferData;

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait Multisigverifier:
  bls_signature::BlsSignatureModule
{
    #[init]
    fn init(&self, bls_pub_keys: MultiValueEncoded<ManagedAddress>) {
        for key in bls_pub_keys {
            self.bls_pub_keys().push(&key);
      }
   }
    
    #[endpoint(registerBridgeOps)]
    fn register_bridge_operations(
        &self,
        hash_of_hashes: ManagedBuffer,
        hash_of_bridge_ops: MultiValueEncoded<ManagedBuffer>,
        signature: BlsSignature<Self::Api>,
        transfer_data: TransferData<Self::Api>
    ) {
        let caller = self.blockchain().get_caller();
        let is_bls_valid = self.verify_bls(hash_of_bridge_ops.clone(), &signature, caller);
        let mut serialized_transferred_data = ManagedBuffer::new();

        if let core::result::Result::Err(err) = transfer_data.top_encode(&mut serialized_transferred_data) {
            sc_panic!("Transfer data encode error: {}", err.message_bytes());
        }

        let transfer_data_sha256 = self.crypto().sha256(&serialized_transferred_data);
        let transfer_data_hash = transfer_data_sha256.as_managed_buffer();
        
        require!(
            hash_of_hashes.eq(transfer_data_hash),
            "Hash of all operations doesn't match the hash of transfer "
        );

        require!(
            is_bls_valid,
            "BLS signature is not valid"
        );

        for hash in hash_of_bridge_ops {
          self.pending_operations_mapper().insert(hash);
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
        let bls_pub_keys = self.bls_pub_keys().len() as u32;

        if is_bls_valid && signatures_count > 2/3 * bls_pub_keys {
            return true
        }

        false
    }

    #[storage_mapper("isValid")]
    fn is_valid(&self) -> SingleValueMapper<bool>;

    #[storage_mapper("bls_pub_keys")]
    fn bls_pub_keys(&self) -> VecMapper<ManagedAddress>;

    #[storage_mapper("signers")]
    fn signatures(&self) -> SingleValueMapper<u32>;

    #[storage_mapper("operations_mapper")]
    fn pending_operations_mapper(&self) -> SetMapper<ManagedBuffer>; 
}
