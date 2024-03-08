#![no_std]

mod field_element;

use field_element::FieldElement;
use multiversx_sc_scenario::num_bigint::BigUint;

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

        self.multi_verify_signature(&serialized_signature_data, signature);

        deserialized_transactions
    }

    fn multi_verify_signature(
        &self,
        signature_data: &ManagedBuffer,
        signature: &BlsSignature<Self::Api>,
    ) {
        let all_signers = self.all_signers();

        let mut total_valid_signatures = 0;
        for signer in all_signers.iter() {
            let is_valid = self.crypto().verify_bls(
                signer.as_managed_buffer(),
                signature_data,
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
    }

    // https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-bls-signature-02#section-3.1
    // https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-10.html#montgomery
    fn bls_fast_aggregate_verify(
        &self,
        signature: &BlsSignature<Self::Api>,
        message: ManagedBuffer,
        count: usize,
        pub_keys: MultiValueEncoded<ManagedBuffer>,
    ) -> bool {
        // aggregate the public keys
        if pub_keys.to_vec().iter().any(|key| key.is_empty()) {
            return false
        }

        let aggregated_pub_key = self.aggregate_pub_keys(pub_keys);

        // hash the message to a point on the curve
        self.hash_and_map_to_g(message, count);

        // calculate the parining and verify the aggregated signature
        false
    }

    fn aggregate_pub_keys(&self, pub_keys: MultiValueEncoded<ManagedBuffer>) -> ManagedBuffer {
        let mut aggregated_pub_key = ManagedBuffer::new();

        for key in pub_keys {
            aggregated_pub_key.append(&key);
        }

        aggregated_pub_key
    }

    fn hash_and_map_to_g(&self, message: ManagedBuffer, count: usize) {
        let mut serialized_message = ManagedBuffer::new();

        if let core::result::Result::Err(err) = message.top_encode(&mut serialized_message) {
            sc_panic!("Message data encoding error: {}", err.message_bytes());
        }

        let sha256 = self.crypto().sha256(&serialized_message);
        let message_hash = sha256.as_managed_buffer();

        // hash -> curve alg
        // hash_to_curve(msg)
        //
        // Input: msg, an arbitrary-length byte string.
        // Output: P, a point in G.
        //
        // Steps:
        self.hash_to_field(message, count);
        // 2. Q0 = map_to_curve(u[0])
        // 3. Q1 = map_to_curve(u[1])
        // 4. R = Q0 + Q1              # Point addition
        // 5. P = clear_cofactor(R)
        // 6. return P
    }

    fn hash_to_field(&self, message: ManagedBuffer, count: usize, prime: &BigUint) -> FieldElement {
        let mut message_hash = self.crypto().sha256(message);

        for i in 0..count {
            // add i to the hash
            //
        }

        let mut big_uint = BigUint::from_bytes_be(message_hash);
        big_uint = big_uint % prime;

        FieldElement { value: big_uint, prime }
    }

    fn map_to_curve(&self, field_element: ManagedBuffer) {
        // map field element to point on the curve
    }

    #[storage_mapper("allSigners")]
    fn all_signers(&self) -> UnorderedSetMapper<ManagedAddress>;

    #[storage_mapper("minValidSigners")]
    fn min_valid_signers(&self) -> SingleValueMapper<u32>;
}
