use structs::Operation;
use proxies::header_verifier_proxy::HeaderverifierProxy;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait ExecuteCommonModule: crate::storage::CrossChainStorage {
    fn calculate_operation_hash(&self, operation: &Operation<Self::Api>) -> ManagedBuffer {
        let mut serialized_data = ManagedBuffer::new();

        if let core::result::Result::Err(err) = operation.top_encode(&mut serialized_data) {
            sc_panic!("Transfer data encode error: {}", err.message_bytes());
        }

        let sha256 = self.crypto().sha256(&serialized_data);
        let hash = sha256.as_managed_buffer().clone();

        hash
    }

    fn lock_operation_hash(&self, operation_hash: &ManagedBuffer, hash_of_hashes: &ManagedBuffer) {
        self.tx()
            .to(self.get_header_verifier_address())
            .typed(HeaderverifierProxy)
            .lock_operation_hash(hash_of_hashes, operation_hash)
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
            "There is no registered Header-Verifier address"
        );

        header_verifier_address_mapper.get()
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
