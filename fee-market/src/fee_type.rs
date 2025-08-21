use error_messages::{
    ERROR_AT_ENCODING, INVALID_FEE, INVALID_FEE_TYPE, INVALID_TOKEN_ID,
    SETUP_PHASE_ALREADY_COMPLETED,
};
use structs::{
    fee::{FeeStruct, FeeType},
    generate_hash::GenerateHash,
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait FeeTypeModule:
    utils::UtilsModule + setup_phase::SetupPhaseModule + custom_events::CustomEventsModule
{
    #[only_owner]
    #[endpoint(removeFeeDuringSetupPhase)]
    fn remove_fee_during_setup_phase(&self, base_token: TokenIdentifier) {
        require!(
            !self.is_setup_phase_complete(),
            SETUP_PHASE_ALREADY_COMPLETED
        );

        self.token_fee(&base_token).clear();
        self.fee_enabled().set(false);
    }

    #[endpoint(removeFee)]
    fn remove_fee(&self, hash_of_hashes: ManagedBuffer, base_token: TokenIdentifier) {
        self.require_setup_complete();

        let token_id_hash = base_token.generate_hash();
        if token_id_hash.is_empty() {
            self.complete_operation(
                &hash_of_hashes,
                &token_id_hash,
                Some(ManagedBuffer::from(ERROR_AT_ENCODING)),
            );
            return;
        };

        self.lock_operation_hash(&hash_of_hashes, &token_id_hash);

        self.token_fee(&base_token).clear();
        self.fee_enabled().set(false);

        self.complete_operation(&hash_of_hashes, &token_id_hash, None);
    }

    #[only_owner]
    #[endpoint(setFeeDuringSetupPhase)]
    fn set_fee_during_setup_phase(&self, fee_struct: FeeStruct<Self::Api>) {
        require!(
            !self.is_setup_phase_complete(),
            SETUP_PHASE_ALREADY_COMPLETED
        );

        if let Some(set_fee_error_msg) = self.set_fee_in_storage(&fee_struct) {
            sc_panic!(set_fee_error_msg);
        }
    }

    #[endpoint(setFee)]
    fn set_fee(&self, hash_of_hashes: ManagedBuffer, fee_struct: FeeStruct<Self::Api>) {
        self.require_setup_complete();

        let fee_hash = fee_struct.generate_hash();
        if fee_hash.is_empty() {
            self.complete_operation(
                &hash_of_hashes,
                &fee_hash,
                Some(ManagedBuffer::from(ERROR_AT_ENCODING)),
            );
            return;
        };

        self.lock_operation_hash(&hash_of_hashes, &fee_hash);

        if let Some(set_fee_error_msg) = self.set_fee_in_storage(&fee_struct) {
            self.complete_operation(
                &hash_of_hashes,
                &fee_hash,
                Some(ManagedBuffer::from(set_fee_error_msg)),
            );
            return;
        }

        self.complete_operation(&hash_of_hashes, &fee_hash, None);
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
