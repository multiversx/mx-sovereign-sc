use multiversx_sc::{
    imports::{SingleValueMapper, UnorderedSetMapper},
    types::ManagedAddress,
};

#[multiversx_sc::module]
pub trait CommonStorage {
    #[storage_mapper]
    fn sov_prefix(&self) -> SingleValueMapper<ManagedBuffer>;

    #[storage_mapper("enshrineEsdtWhitelist")]
    fn enshrine_esdt_whitelist(&self) -> UnorderedSetMapper<ManagedAddress<Self::Api>>;
}
