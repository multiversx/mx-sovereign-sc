use multiversx_sc::imports::*;

#[multiversx_sc::module]
pub trait CommonStorage {
    #[storage_mapper("wegldIdentifier")]
    fn wegld_identifier(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("sovereignTokensPrefix")]
    fn sovereign_tokens_prefix(&self) -> SingleValueMapper<ManagedBuffer>;

    #[storage_mapper("tokenHandlerAddress")]
    fn token_handler_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("paidIssuedTokens")]
    fn paid_issued_tokens(&self) -> UnorderedSetMapper<TokenIdentifier<Self::Api>>;
}
