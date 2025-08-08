multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait ExecuteCommonModule: crate::storage::CrossChainStorage {
    fn is_native_token(&self, token_identifier: &TokenIdentifier) -> bool {
        let esdt_safe_native_token_mapper = self.native_token();

        if esdt_safe_native_token_mapper.is_empty() {
            return false;
        }

        token_identifier == &esdt_safe_native_token_mapper.get()
    }

    #[inline]
    fn is_fungible(self, token_type: &EsdtTokenType) -> bool {
        *token_type == EsdtTokenType::Fungible
    }

    #[inline]
    fn is_sft_or_meta(self, token_type: &EsdtTokenType) -> bool {
        *token_type == EsdtTokenType::SemiFungible
            || *token_type == EsdtTokenType::DynamicSFT
            || *token_type == EsdtTokenType::MetaFungible
            || *token_type == EsdtTokenType::DynamicMeta
    }

    #[inline]
    fn is_nft(self, token_type: &EsdtTokenType) -> bool {
        *token_type == EsdtTokenType::NonFungible
            || *token_type == EsdtTokenType::NonFungibleV2
            || *token_type == EsdtTokenType::DynamicNFT
    }
}
