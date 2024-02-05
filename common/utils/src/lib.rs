#![no_std]

use bls_signature::BlsSignature;

multiversx_sc::imports!();

pub type PaymentsVec<M> = ManagedVec<M, EsdtTokenPayment<M>>;

static ERR_EMPTY_PAYMENTS: &[u8] = b"No payments";

#[multiversx_sc::module]
pub trait UtilsModule: bls_signature::BlsSignatureModule {
    fn require_sc_address(&self, address: &ManagedAddress) {
        require!(
            !address.is_zero() && self.blockchain().is_smart_contract(address),
            "Invalid SC address"
        );
    }

    fn require_valid_token_id(&self, token_id: &TokenIdentifier) {
        require!(token_id.is_valid_esdt_identifier(), "Invalid token ID");
    }

    fn remove_items<
        T: TopEncode + TopDecode + NestedEncode + NestedDecode + 'static,
        I: IntoIterator<Item = T>,
    >(
        &self,
        mapper: &mut UnorderedSetMapper<T>,
        items: I,
    ) {
        for item in items {
            let was_removed = mapper.swap_remove(&item);
            require!(was_removed, "Item not found in list");
        }
    }

    fn pop_first_payment(
        &self,
        payments: &mut PaymentsVec<Self::Api>,
    ) -> EsdtTokenPayment<Self::Api> {
        require!(!payments.is_empty(), ERR_EMPTY_PAYMENTS);

        let first_payment = payments.get(0);
        payments.remove(0);

        first_payment
    }

    fn verify_items_signature<T: TopDecode + NestedEncode + ManagedVecItem>(
        &self,
        opt_signature: Option<BlsSignature<Self::Api>>,
        items: MultiValueEncoded<T>,
    ) -> ManagedVec<T> {
        require!(opt_signature.is_some(), "Must provide signature");

        let list = items.to_vec();
        let signature = unsafe { opt_signature.unwrap_unchecked() };
        let mut signature_data = ManagedBuffer::new();
        for token in &list {
            let _ = token.dep_encode(&mut signature_data);
        }

        self.multi_verify_signature(&signature_data, &signature);

        list
    }
}
