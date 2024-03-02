#![no_std]

use bls_signature::BlsSignature;

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
    
    #[endpoint]
    fn upgrade(&self) {}

    #[endpoint(registerBridgeOps)]
    fn register_bridge_operations(
        &self,
        bridge_operations_hash: ManagedBuffer,
        operations_hashes: ManagedVec<ManagedBuffer>,
        signature: BlsSignature<Self::Api>,
    ) {
        let caller = self.blockchain().get_caller();
        let is_bls_valid = self.verify_bls(&signature, caller, &bridge_operations_hash);

        require!(
            is_bls_valid,
            "BLS signature is not valid"
        );
        
        self.calculate_and_check_transfers_hashes(&bridge_operations_hash, operations_hashes.clone());

        for operation in &operations_hashes {
            self.pending_hashes().insert(operation);
        }
    }

    fn calculate_and_check_transfers_hashes(
        &self,
        transfers_hash: &ManagedBuffer,
        transfers_data: ManagedVec<ManagedBuffer>
    ) {
        let mut transfers_hashes = ManagedBuffer::new();

        for transfer in &transfers_data {
            let transfer_sha256 = self.crypto().sha256(&transfer);
            let transfer_hash = transfer_sha256.as_managed_buffer();

            transfers_hashes.append(transfer_hash);
        }

        let hash_of_hashes_sha256 = self.crypto().sha256(&transfers_hashes); 
        let hash_of_hashes = hash_of_hashes_sha256.as_managed_buffer();

        require!(
            transfers_hash.eq(hash_of_hashes),
            "Hash of all operations doesn't match the hash of transfer data"
        );
    }

    fn verify_bls(
        &self,
        signature: &BlsSignature<Self::Api>,
        user: ManagedAddress,
        bridge_operations_hash: &ManagedBuffer
    ) -> bool {
        let mut serialized_signature_data = ManagedBuffer::new();

        let _ = bridge_operations_hash.dep_encode(&mut serialized_signature_data);

        let is_bls_valid = self.crypto().verify_bls(
            user.as_managed_buffer(), 
            &serialized_signature_data, 
            signature.as_managed_buffer()
        );

        self.is_signature_valid_and_approved(is_bls_valid)
    }

    fn is_signature_valid_and_approved(&self, is_bls_valid: bool) -> bool {
        let signatures_count = self.signatures().get();
        let bls_pub_keys = self.bls_pub_keys().len() as u32;
        let minimum_signatures = 2 * bls_pub_keys / 3;

        is_bls_valid && signatures_count > minimum_signatures 
    }

    #[storage_mapper("bls_pub_keys")]
    fn bls_pub_keys(&self) -> VecMapper<ManagedAddress>;

    #[storage_mapper("signatures")]
    fn signatures(&self) -> SingleValueMapper<u32>;

    #[storage_mapper("pending_hashes")]
    fn pending_hashes(&self) -> UnorderedSetMapper<ManagedBuffer>; 
}
