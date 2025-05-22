use error_messages::NO_HEADER_VERIFIER_ADDRESS;
use proxies::header_verifier_proxy::HeaderverifierProxy;
use structs::operation::Operation;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait ExecuteCommonModule: crate::storage::CrossChainStorage {
    fn lock_operation_hash(&self, hash_of_hashes: &ManagedBuffer, hash: &ManagedBuffer) {
        self.tx()
            .to(self.get_header_verifier_address())
            .typed(HeaderverifierProxy)
            .lock_operation_hash(hash_of_hashes, hash)
            .sync_call();
    }

    fn remove_executed_hash(&self, hash_of_hashes: &ManagedBuffer, op_hash: &ManagedBuffer) {
        self.tx()
            .to(self.get_header_verifier_address())
            .typed(HeaderverifierProxy)
            .remove_executed_hash(hash_of_hashes, op_hash)
            .sync_call();
    }

    fn get_header_verifier_address(&self) -> ManagedAddress {
        let header_verifier_address_mapper = self.header_verifier_address();

        require!(
            !header_verifier_address_mapper.is_empty(),
            NO_HEADER_VERIFIER_ADDRESS
        );

        header_verifier_address_mapper.get()
    }

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
            || *token_type == EsdtTokenType::Meta
            || *token_type == EsdtTokenType::DynamicMeta
    }

    #[inline]
    fn is_nft(self, token_type: &EsdtTokenType) -> bool {
        *token_type == EsdtTokenType::NonFungible || *token_type == EsdtTokenType::DynamicNFT
    }
}
