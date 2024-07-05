use multiversx_sc::imports::*;

#[multiversx_sc::module]
pub trait CommonStorage {
    #[storage_mapper("isSovereignChain")]
    fn is_sovereign_chain(&self) -> SingleValueMapper<bool>;

    #[storage_mapper("wegldIdentifier")]
    fn wegld_identifier(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("sovereignTokensPrefix")]
    fn sovereign_tokens_prefix(&self) -> SingleValueMapper<ManagedBuffer>;
}
