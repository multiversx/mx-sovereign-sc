multiversx_sc::imports!();

mod bridge_proxy {
    multiversx_sc::imports!();

    #[multiversx_sc::proxy]
    pub trait BridgeProxy {
        #[init]
        fn init(&self, min_valid_signers: u32, signers: MultiValueEncoded<ManagedAddress>);
    }
}

#[multiversx_sc::module]
pub trait BridgeModule {
    #[only_owner]
    #[endpoint(deployBridge)]
    fn deploy_bridge(
        &self,
        code: ManagedBuffer,
        min_valid_signers: u32,
        signers: MultiValueEncoded<ManagedAddress>,
    ) {
        require!(self.bridge_address().is_empty(), "Bridge already deployed");

        let metadata =
            CodeMetadata::PAYABLE_BY_SC | CodeMetadata::UPGRADEABLE | CodeMetadata::READABLE;
        let (sc_address, _) = self
            .bridge_proxy()
            .init(min_valid_signers, signers)
            .deploy_contract::<IgnoreValue>(&code, metadata);

        self.bridge_address().set(sc_address);
    }

    #[proxy]
    fn bridge_proxy(&self) -> bridge_proxy::Proxy<Self::Api>;

    #[storage_mapper("bridgeAddress")]
    fn bridge_address(&self) -> SingleValueMapper<ManagedAddress>;
}
