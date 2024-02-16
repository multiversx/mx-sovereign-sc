#![no_std]

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
        bridge_operations_hash: ManagedBuffer,
        signature: BlsSignature<Self::Api>,
        bridge_operations: MultiValueEncoded<TransferData<Self::Api>>
    ) {
        let caller = self.blockchain().get_caller();
        let is_bls_valid = self.verify_bls(&signature, caller, bridge_operations.clone());

        require!(
            is_bls_valid,
            "BLS signature is not valid"
        );

        let mut serialized_transferred_data = ManagedBuffer::new();

        for operation in bridge_operations {
            if let core::result::Result::Err(err) = operation.top_encode(&mut serialized_transferred_data) {
                sc_panic!("Transfer data encode error: {}", err.message_bytes());
            }
        }

        let transfers_data_sha256 = self.crypto().sha256(&serialized_transferred_data);
        let transfers_data_hash = transfers_data_sha256.as_managed_buffer();
        
        require!(
            bridge_operations_hash.eq(transfers_data_hash),
            "Hash of all operations doesn't match the hash of transfer data"
        );

        self.pending_operations().insert(bridge_operations_hash);
    }

    fn verify_bls (
        &self,
        signature: &BlsSignature<Self::Api>,
        user: ManagedAddress,
        bridge_operations: MultiValueEncoded<TransferData<Self::Api>>,
    ) -> bool {
        let mut serialized_signature_data = ManagedBuffer::new();

        for operation in bridge_operations.into_iter() {
            let _ = operation.dep_encode(&mut serialized_signature_data);
        }

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

    #[storage_mapper("operations_mapper")]
    fn pending_operations(&self) -> UnorderedSetMapper<ManagedBuffer>; 
}
