#![no_std]

use error_messages::{
    ERROR_AT_ENCODING, ERR_EMPTY_PAYMENTS, INVALID_PREFIX, INVALID_SC_ADDRESS, TOKEN_ID_NO_PREFIX,
};
use proxies::header_verifier_proxy::HeaderverifierProxy;
use structs::aliases::PaymentsVec;

multiversx_sc::imports!();

const DASH: u8 = b'-';
const MAX_TOKEN_ID_LEN: usize = 32;
const MIN_PREFIX_LENGTH: usize = 1;
const MAX_PREFIX_LENGTH: usize = 4;

#[multiversx_sc::module]
pub trait UtilsModule: custom_events::CustomEventsModule {
    fn lock_operation_hash_wrapper(&self, hash_of_hashes: &ManagedBuffer, hash: &ManagedBuffer) {
        self.tx()
            .to(self.blockchain().get_owner_address())
            .typed(HeaderverifierProxy)
            .lock_operation_hash(hash_of_hashes, hash)
            .sync_call();
    }

    fn remove_executed_hash_wrapper(
        &self,
        hash_of_hashes: &ManagedBuffer,
        op_hash: &ManagedBuffer,
    ) {
        self.tx()
            .to(self.blockchain().get_owner_address())
            .typed(HeaderverifierProxy)
            .remove_executed_hash(hash_of_hashes, op_hash)
            .sync_call();
    }

    fn complete_operation(
        &self,
        hash_of_hashes: &ManagedBuffer,
        operation_hash: &ManagedBuffer,
        error_message: Option<ManagedBuffer>,
    ) {
        self.execute_bridge_operation_event(hash_of_hashes, operation_hash, error_message);
        self.remove_executed_hash_wrapper(hash_of_hashes, operation_hash);
    }

    fn require_sc_address(&self, address: &ManagedAddress) {
        require!(
            !address.is_zero() && self.blockchain().is_smart_contract(address),
            INVALID_SC_ADDRESS
        );
    }

    fn is_valid_token_id(&self, token_id: &EgldOrEsdtTokenIdentifier<Self::Api>) -> bool {
        token_id.clone().unwrap_esdt().is_valid_esdt_identifier()
    }

    fn pop_first_payment(
        &self,
        payments: PaymentsVec<Self::Api>,
    ) -> MultiValue2<OptionalValue<EgldOrEsdtTokenPayment<Self::Api>>, PaymentsVec<Self::Api>> {
        require!(!payments.is_empty(), ERR_EMPTY_PAYMENTS);

        let mut new_payments = payments;

        let first_payment = new_payments.get(0).clone();
        new_payments.remove(0);

        MultiValue2::from((OptionalValue::Some(first_payment.clone()), new_payments))
    }

    fn has_prefix(&self, token_id: &EgldOrEsdtTokenIdentifier<Self::Api>) -> bool {
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
    fn require_token_has_prefix(&self, token_id: &EgldOrEsdtTokenIdentifier<Self::Api>) {
        require!(self.has_prefix(token_id), TOKEN_ID_NO_PREFIX);
    }

    fn has_sov_prefix(
        &self,
        token_id: &EgldOrEsdtTokenIdentifier<Self::Api>,
        chain_prefix: &ManagedBuffer,
    ) -> bool {
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

    fn validate_operation_hash(&self, hash: &ManagedBuffer) -> Option<ManagedBuffer> {
        if hash.is_empty() {
            return Some(ERROR_AT_ENCODING.into());
        }

        None
    }

    fn require_valid_sov_token_prefix(&self, sov_token_prefix: &ManagedBuffer) {
        let prefix_len = sov_token_prefix.len();
        require!(
            prefix_len > MIN_PREFIX_LENGTH && prefix_len <= MAX_PREFIX_LENGTH,
            INVALID_PREFIX
        );
    }
}
