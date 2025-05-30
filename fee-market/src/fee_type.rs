use error_messages::{INVALID_FEE, INVALID_FEE_TYPE, INVALID_TOKEN_ID};
use structs::{
    fee::{FeeStruct, FeeType},
    generate_hash::GenerateHash,
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait FeeTypeModule:
    utils::UtilsModule + setup_phase::SetupPhaseModule + events::EventsModule
{
    #[only_owner]
    #[endpoint(removeFee)]
    fn remove_fee_during_setup_phase(&self, base_token: TokenIdentifier) {
        self.token_fee(&base_token).clear();
        self.fee_enabled().set(false);
    }

    #[only_owner]
    #[endpoint(removeFee)]
    fn remove_fee(&self, hash_of_hashes: ManagedBuffer, base_token: TokenIdentifier) {
        self.require_setup_complete();

        let token_id_hash = base_token.generate_hash();
        self.lock_operation_hash(&hash_of_hashes, &token_id_hash);

        self.token_fee(&base_token).clear();
        self.fee_enabled().set(false);

        self.remove_executed_hash(&hash_of_hashes, &token_id_hash);
    }

    #[only_owner]
    #[endpoint(setFee)]
    fn set_fee_during_setup_phase(&self, fee_struct: FeeStruct<Self::Api>) {
        if let Some(set_fee_error_msg) = self.set_fee_in_storage(&fee_struct) {
            sc_panic!(set_fee_error_msg);
        }
    }

    #[only_owner]
    #[endpoint(setFee)]
    fn set_fee(&self, hash_of_hashes: ManagedBuffer, fee_struct: FeeStruct<Self::Api>) {
        self.require_setup_complete();

        let fee_hash = fee_struct.generate_hash();
        self.lock_operation_hash(&hash_of_hashes, &fee_hash);

        if let Some(set_fee_error_msg) = self.set_fee_in_storage(&fee_struct) {
            self.failed_bridge_operation_event(
                &hash_of_hashes,
                &fee_hash,
                &ManagedBuffer::from(set_fee_error_msg),
            );
            self.remove_executed_hash(&hash_of_hashes, &fee_hash);

            return;
        }

        self.remove_executed_hash(&hash_of_hashes, &fee_hash);
        self.execute_bridge_operation_event(&hash_of_hashes, &fee_hash);
    }

    fn set_fee_in_storage(&self, fee_struct: &FeeStruct<Self::Api>) -> Option<&str> {
        if !self.is_valid_token_id(&fee_struct.base_token) {
            return Some(INVALID_TOKEN_ID);
        }

        let token = match &fee_struct.fee_type {
            FeeType::None => sc_panic!(INVALID_FEE_TYPE),
            FeeType::Fixed {
                token,
                per_transfer: _,
                per_gas: _,
            } => {
                require!(&fee_struct.base_token == token, INVALID_FEE);

                token
            }
            FeeType::AnyToken {
                base_fee_token,
                per_transfer: _,
                per_gas: _,
            } => base_fee_token,
        };

        if !self.is_valid_token_id(token) {
            return Some(INVALID_TOKEN_ID);
        }

        self.fee_enabled().set(true);
        self.token_fee(&fee_struct.base_token)
            .set(fee_struct.fee_type.clone());

        None
    }

    fn is_fee_enabled(&self) -> bool {
        self.fee_enabled().get()
    }

    #[view(getTokenFee)]
    #[storage_mapper("tokenFee")]
    fn token_fee(&self, token_id: &TokenIdentifier) -> SingleValueMapper<FeeType<Self::Api>>;

    #[storage_mapper("feeEnabledFlag")]
    fn fee_enabled(&self) -> SingleValueMapper<bool>;
}
