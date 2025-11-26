use error_messages::{BURN_ESDT_FAILED, MINT_ESDT_FAILED};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait ExecuteCommonModule: crate::storage::CrossChainStorage {
    fn is_native_token(&self, token_identifier: &EgldOrEsdtTokenIdentifier<Self::Api>) -> bool {
        let esdt_safe_native_token_mapper = self.native_token();

        if esdt_safe_native_token_mapper.is_empty() {
            return false;
        }

        token_identifier == &esdt_safe_native_token_mapper.get()
    }

    #[inline]
    fn format_error(
        &self,
        error: &str,
        token_id: EsdtTokenIdentifier,
        error_code: u32,
    ) -> ManagedBuffer<Self::Api> {
        let prefix: ManagedBuffer = error.into();
        let error_message = sc_format!("{} {}; error code: {}", prefix, token_id, error_code);

        error_message
    }

    #[inline]
    fn try_esdt_local_burn(
        &self,
        token_id: &EsdtTokenIdentifier<Self::Api>,
        token_nonce: u64,
        amount: &BigUint<Self::Api>,
    ) -> Result<(), ManagedBuffer<Self::Api>> {
        let result = self
            .tx()
            .to(ToSelf)
            .typed(UserBuiltinProxy)
            .esdt_local_burn(token_id, token_nonce, amount)
            .returns(ReturnsHandledOrError::new())
            .sync_call_fallible();

        result
            .map_err(|error_code| self.format_error(BURN_ESDT_FAILED, token_id.clone(), error_code))
    }

    #[inline]
    fn try_esdt_local_mint(
        &self,
        token_id: &EsdtTokenIdentifier<Self::Api>,
        token_nonce: u64,
        amount: &BigUint<Self::Api>,
    ) -> Result<(), ManagedBuffer<Self::Api>> {
        let result = self
            .tx()
            .to(ToSelf)
            .typed(UserBuiltinProxy)
            .esdt_local_mint(token_id, token_nonce, amount)
            .returns(ReturnsHandledOrError::new())
            .sync_call_fallible();

        result
            .map_err(|error_code| self.format_error(MINT_ESDT_FAILED, token_id.clone(), error_code))
    }

    #[inline]
    fn is_fungible(&self, token_type: &EsdtTokenType) -> bool {
        *token_type == EsdtTokenType::Fungible
    }

    #[inline]
    fn is_sft_or_meta(&self, token_type: &EsdtTokenType) -> bool {
        *token_type == EsdtTokenType::SemiFungible
            || *token_type == EsdtTokenType::DynamicSFT
            || *token_type == EsdtTokenType::MetaFungible
            || *token_type == EsdtTokenType::DynamicMeta
    }

    #[inline]
    fn is_nft(&self, token_type: &EsdtTokenType) -> bool {
        *token_type == EsdtTokenType::NonFungible
            || *token_type == EsdtTokenType::NonFungibleV2
            || *token_type == EsdtTokenType::DynamicNFT
    }
}
