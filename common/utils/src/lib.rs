#![no_std]

use error_messages::{
    ERR_EMPTY_PAYMENTS, INVALID_SC_ADDRESS, INVALID_TOKEN_ID, ITEM_NOT_IN_LIST, TOKEN_ID_NO_PREFIX,
};
use structs::aliases::PaymentsVec;

multiversx_sc::imports!();

const DASH: u8 = b'-';
const MAX_TOKEN_ID_LEN: usize = 32;

#[multiversx_sc::module]
pub trait UtilsModule {
    fn require_sc_address(&self, address: &ManagedAddress) {
        require!(
            !address.is_zero() && self.blockchain().is_smart_contract(address),
            INVALID_SC_ADDRESS
        );
    }

    fn require_valid_token_id(&self, token_id: &TokenIdentifier) {
        require!(token_id.is_valid_esdt_identifier(), INVALID_TOKEN_ID);
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
            require!(was_removed, ITEM_NOT_IN_LIST);
        }
    }

    fn pop_first_payment(
        &self,
        payments: PaymentsVec<Self::Api>,
    ) -> MultiValue2<OptionalValue<EsdtTokenPayment<Self::Api>>, PaymentsVec<Self::Api>> {
        require!(!payments.is_empty(), ERR_EMPTY_PAYMENTS);

        let mut new_payments = payments;

        let first_payment = new_payments.get(0).clone();
        new_payments.remove(0);

        MultiValue2::from((OptionalValue::Some(first_payment.clone()), new_payments))
    }

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

    #[inline]
    fn require_token_has_prefix(&self, token_id: &TokenIdentifier) {
        require!(self.has_prefix(token_id), TOKEN_ID_NO_PREFIX);
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
