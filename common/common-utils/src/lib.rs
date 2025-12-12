#![no_std]

use error_messages::{
    CHAIN_ID_NOT_LOWERCASE_ALPHANUMERIC, ERROR_AT_GENERATING_OPERATION_HASH, ERR_EMPTY_PAYMENTS,
    INVALID_CHAIN_ID, INVALID_SC_ADDRESS, TOKEN_ID_NO_PREFIX,
};
use proxies::header_verifier_proxy::HeaderverifierProxy;
use structs::aliases::PaymentsVec;

multiversx_sc::imports!();

const DASH: u8 = b'-';
const MAX_TOKEN_ID_LEN: usize = 32;
const MIN_PREFIX_LENGTH: usize = 1;
const MAX_PREFIX_LENGTH: usize = 4;
const CHARSET: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";

#[multiversx_sc::module]
pub trait CommonUtilsModule: custom_events::CustomEventsModule {
    fn lock_operation_hash_wrapper(
        &self,
        hash_of_hashes: &ManagedBuffer,
        hash: &ManagedBuffer,
        nonce: u64,
    ) -> Option<ManagedBuffer> {
        self.tx()
            .to(self.blockchain().get_owner_address())
            .typed(HeaderverifierProxy)
            .lock_operation_hash(hash_of_hashes, hash, nonce)
            .returns(ReturnsResult)
            .sync_call()
            .into_option()
    }

    fn remove_executed_hash_wrapper(
        &self,
        hash_of_hashes: &ManagedBuffer,
        op_hash: &ManagedBuffer,
    ) -> Option<ManagedBuffer> {
        self.tx()
            .to(self.blockchain().get_owner_address())
            .typed(HeaderverifierProxy)
            .remove_executed_hash(hash_of_hashes, op_hash)
            .returns(ReturnsResult)
            .sync_call()
            .into_option()
    }

    fn complete_operation(
        &self,
        hash_of_hashes: &ManagedBuffer,
        operation_hash: &ManagedBuffer,
        error_message: Option<ManagedBuffer>,
    ) {
        let remove_error = self.remove_executed_hash_wrapper(hash_of_hashes, operation_hash);

        let merged_error = match (error_message, remove_error) {
            (None, None) => None,
            (Some(err), None) | (None, Some(err)) => Some(err),
            (Some(err1), Some(err2)) => {
                let mut errors: ManagedVec<Self::Api, ManagedBuffer> = ManagedVec::new();
                errors.push(err1);
                errors.push(err2);
                Some(self.combine_error_messages(&errors))
            }
        };

        self.execute_bridge_operation_event(hash_of_hashes, operation_hash, merged_error);
    }

    fn require_sc_address(&self, address: &ManagedAddress) {
        require!(
            !address.is_zero() && self.blockchain().is_smart_contract(address),
            INVALID_SC_ADDRESS
        );
    }

    fn is_valid_token_id(&self, token_id: &EgldOrEsdtTokenIdentifier<Self::Api>) -> bool {
        token_id.clone().is_valid()
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
            return Some(ERROR_AT_GENERATING_OPERATION_HASH.into());
        }

        None
    }

    fn is_chain_id_lowercase_alphanumeric(&self, chain_id: &ManagedBuffer) -> bool {
        let mut chain_id_byte_array = [0u8; 4];
        let chain_id_byte_array = chain_id.load_to_byte_array(&mut chain_id_byte_array);

        chain_id_byte_array.iter().all(|b| CHARSET.contains(b))
    }

    fn combine_error_messages(
        &self,
        errors: &ManagedVec<Self::Api, ManagedBuffer>,
    ) -> ManagedBuffer {
        let newline: ManagedBuffer = "\n".into();
        let mut aggregated = ManagedBuffer::new();

        for (index, error_message) in errors.iter().enumerate() {
            if index > 0 {
                aggregated.append(&newline);
            }
            aggregated.append(&error_message);
        }

        aggregated
    }

    #[inline]
    fn validate_chain_id(&self, chain_id: &ManagedBuffer) {
        let id_length = chain_id.len();
        require!(
            (MIN_PREFIX_LENGTH..=MAX_PREFIX_LENGTH).contains(&id_length),
            INVALID_CHAIN_ID
        );

        require!(
            self.is_chain_id_lowercase_alphanumeric(chain_id),
            CHAIN_ID_NOT_LOWERCASE_ALPHANUMERIC
        );
    }
}
