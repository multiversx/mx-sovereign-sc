use transaction::Transaction;

multiversx_sc::imports!();

pub const BLS_SIGNATURE_LEN: usize = 48;
pub type BlsSignature<M> = ManagedByteArray<M, BLS_SIGNATURE_LEN>;

#[multiversx_sc::module]
pub trait BlsSignatureModule {
    #[only_owner]
    #[endpoint(setMinValidSigners)]
    fn set_min_valid_signers(&self, new_value: u32) {
        self.min_valid_signers().set(new_value);
    }

    #[only_owner]
    #[endpoint(addSigners)]
    fn add_signers(&self, signers: MultiValueEncoded<ManagedAddress>) {
        for signer in signers {
            let _ = self.all_signers().insert(signer);
        }
    }

    #[only_owner]
    #[endpoint(removeSigners)]
    fn remove_signers(&self, signers: MultiValueEncoded<ManagedAddress>) {
        for signer in signers {
            let _ = self.all_signers().swap_remove(&signer);
        }
    }

    fn verify_bls_signature(
        &self,
        transactions: MultiValueEncoded<Transaction<Self::Api>>,
        signature: &BlsSignature<Self::Api>,
    ) -> ManagedVec<Transaction<Self::Api>> {
        let mut deserialized_transactions = ManagedVec::new();
        let mut serialized_signature_data = ManagedBuffer::new();
        for transaction in transactions {
            let _ = transaction.dep_encode(&mut serialized_signature_data);
            
            deserialized_transactions.push(transaction);
        }

        let all_signers = self.all_signers();

        let mut total_valid_signatures = 0;
        for signer in all_signers.iter() {
            let is_valid = self.crypto().verify_bls(
                signer.as_managed_buffer(),
                &serialized_signature_data,
                signature.as_managed_buffer(),
            );
            if is_valid {
                total_valid_signatures += 1;
            }
        }

        let min_valid_signers = self.min_valid_signers().get();
        require!(
            total_valid_signatures >= min_valid_signers,
            "Invalid signature"
        );

        deserialized_transactions
    }

    #[storage_mapper("allSigners")]
    fn all_signers(&self) -> UnorderedSetMapper<ManagedAddress>;

    #[storage_mapper("minValidSigners")]
    fn min_valid_signers(&self) -> SingleValueMapper<u32>;
}
