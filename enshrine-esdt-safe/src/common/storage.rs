use multiversx_sc::imports::*;

#[multiversx_sc::module]
pub trait CommonStorage {
    #[storage_mapper("isSovereignChain")]
    fn is_sovereign_chain(&self) -> SingleValueMapper<bool>;

    #[storage_mapper("wegldTicker")]
    fn wegld_ticker(&self) -> SingleValueMapper<TokenIdentifier>;
}
