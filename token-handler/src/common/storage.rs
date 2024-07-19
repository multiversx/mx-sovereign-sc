use multiversx_sc::imports::SingleValueMapper;

#[multiversx_sc::module]
pub trait CommonStorage {
    #[storage_mapper]
    fn sov_prefix(&self) -> SingleValueMapper<ManagedBuffer>;

    #[storage_mapper]
    fn header_verifier_address(&self) -> SingleValueMapper<ManagedAddress>;
}
