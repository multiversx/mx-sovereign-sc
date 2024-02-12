multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait UtilsModule {

    //proxies
    #[proxy]
    fn esdt_safe_proxy(&self, sc_address: ManagedAddress) -> esdt_safe::Proxy<Self::Api>;

    fn get_esdt_safe_proxy_instance(&self, proxy_address: ManagedAddress) -> esdt_safe::Proxy<Self::Api> {
        self.esdt_safe_proxy(proxy_address)
    }
}
