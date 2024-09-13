multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode, NestedDecode)]
pub enum FeeType<M: ManagedTypeApi> {
    None,
    Fixed {
        token: TokenIdentifier<M>,
        per_transfer: BigUint<M>,
        per_gas: BigUint<M>,
    },
    AnyToken {
        base_fee_token: TokenIdentifier<M>,
        per_transfer: BigUint<M>,
        per_gas: BigUint<M>,
    },
}

#[type_abi]
#[derive(NestedDecode)]
pub struct FeeStruct<M: ManagedTypeApi> {
    pub base_token: TokenIdentifier<M>,
    pub fee_type: FeeType<M>,
}

#[multiversx_sc::module]
pub trait FeeTypeModule: utils::UtilsModule + bls_signature::BlsSignatureModule {
    #[only_owner]
    #[endpoint(addFee)]
    fn add_fee(&self, base_token: TokenIdentifier, fee_type: FeeType<Self::Api>) {
        self.require_valid_token_id(&base_token);

        let token = match &fee_type {
            FeeType::None => sc_panic!("Invalid fee type"),
            FeeType::Fixed {
                token,
                per_transfer: _,
                per_gas: _,
            } => {
                require!(&base_token == token, "Invalid fee");

                token
            }
            FeeType::AnyToken {
                base_fee_token,
                per_transfer: _,
                per_gas: _,
            } => base_fee_token,
        };

        self.require_valid_token_id(token);
        self.is_fee_enabled().set(true);
        self.token_fee(&base_token).set(fee_type);
    }

    fn is_fee_enabled(&self) -> bool {
        self.fee_enabled().get()
    }

    #[only_owner]
    #[endpoint(removeFee)]
    fn remove_fee(&self, base_token: TokenIdentifier) {
        self.token_fee(&base_token).clear();
    }

    #[view(getTokenFee)]
    #[storage_mapper("tokenFee")]
    fn token_fee(&self, token_id: &TokenIdentifier) -> SingleValueMapper<FeeType<Self::Api>>;

    #[storage_mapper("feeEnabledFlag")]
    fn fee_enabled(&self) -> SingleValueMapper<bool>;
}
