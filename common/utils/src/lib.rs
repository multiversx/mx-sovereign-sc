#![no_std]

multiversx_sc::imports!();

pub type PaymentsVec<M> = ManagedVec<M, EsdtTokenPayment<M>>;

static ERR_EMPTY_PAYMENTS: &[u8] = b"No payments";
const DASH: u8 = b'-';
const MAX_TOKEN_ID_LEN: usize = 32;

#[multiversx_sc::module]
pub trait UtilsModule {
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

    // fn verify_items_signature<T: TopDecode + NestedEncode + ManagedVecItem>(
    //     &self,
    //     opt_signature: Option<BlsSignature<Self::Api>>,
    //     items: MultiValueEncoded<T>,
    // ) -> ManagedVec<T> {
    //     require!(opt_signature.is_some(), "Must provide signature");
    //
    //     let list = items.to_vec();
    //     let signature = unsafe { opt_signature.unwrap_unchecked() };
    //     let mut signature_data = ManagedBuffer::new();
    //     for token in &list {
    //         let _ = token.dep_encode(&mut signature_data);
    //     }
    //
    //     self.multi_verify_signature(&signature_data, &signature);
    //
    //     list
    // }

    fn has_prefix(&self, token_id: &TokenIdentifier) -> bool {
        let buffer = token_id.as_managed_buffer();
        let mut array_buffer = [0u8; MAX_TOKEN_ID_LEN];
        let slice = buffer.load_to_byte_array(&mut array_buffer);

        let counter = slice.iter().filter(|&&c| c == DASH).count();

        if counter == 2 {
            return true;
        }

        false
    }

    fn has_sov_prefix(&self, token_id: &TokenIdentifier, chain_prefix: &ManagedBuffer) -> bool {
        if !self.has_prefix(token_id) {
            return false;
        }

        let buffer = token_id.as_managed_buffer();
        let mut array_buffer = [0u8; MAX_TOKEN_ID_LEN];
        let slice = buffer.load_to_byte_array(&mut array_buffer);

        if let Some(index) = slice.iter().position(|&b| b == DASH) {
            let prefix = ManagedBuffer::from(&slice[..index]);

            if prefix == chain_prefix.clone() {
                return true;
            }
        }

        false
    }
}
