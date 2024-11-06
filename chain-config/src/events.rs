multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait EventsModule {
    #[event("registerEvent")]
    fn register_event(
        &self,
        #[indexed] address: &ManagedAddress,
        #[indexed] bls_keys: &MultiValueEncoded<ManagedBuffer>,
    );
}
