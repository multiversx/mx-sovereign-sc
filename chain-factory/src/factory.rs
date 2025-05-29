use multiversx_sc::imports::*;
use multiversx_sc_modules::only_admin;
use proxies::{
    chain_config_proxy::ChainConfigContractProxy, enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy,
    fee_market_proxy::FeeMarketProxy, header_verifier_proxy::HeaderverifierProxy,
    mvx_esdt_safe_proxy::MvxEsdtSafeProxy,
};
use structs::{
    configs::{EsdtSafeConfig, SovereignConfig},
    fee::FeeStruct,
    forge::ContractInfo,
};
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait FactoryModule: only_admin::OnlyAdminModule {
    #[only_admin]
    #[endpoint(deploySovereignChainConfigContract)]
    fn deploy_sovereign_chain_config_contract(
        &self,
        opt_config: OptionalValue<SovereignConfig<Self::Api>>,
    ) -> ManagedAddress {
        let source_address = self.chain_config_template().get();
        let metadata = self.blockchain().get_code_metadata(&source_address);

        self.tx()
            .typed(ChainConfigContractProxy)
            .init(opt_config)
            .gas(60_000_000)
            .from_source(source_address)
            .code_metadata(metadata)
            .returns(ReturnsNewManagedAddress)
            .sync_call()
    }

    #[only_admin]
    #[endpoint(deployHeaderVerifier)]
    fn deploy_header_verifier(
        &self,
        sovereign_contracts: MultiValueEncoded<ContractInfo<Self::Api>>,
    ) -> ManagedAddress {
        let source_address = self.header_verifier_template().get();
        let metadata = self.blockchain().get_code_metadata(&source_address);

        self.tx()
            .typed(HeaderverifierProxy)
            .init(sovereign_contracts)
            .gas(60_000_000)
            .from_source(source_address)
            .code_metadata(metadata)
            .returns(ReturnsNewManagedAddress)
            .sync_call()
    }

    #[only_admin]
    #[endpoint(deployEnshrineEsdtSafe)]
    fn deploy_enshrine_esdt_safe(
        &self,
        is_sovereign_chain: bool,
        token_handler_address: ManagedAddress,
        wegld_identifier: TokenIdentifier,
        sov_token_prefix: ManagedBuffer,
        opt_config: Option<EsdtSafeConfig<Self::Api>>,
    ) -> ManagedAddress {
        let source_address = self.mvx_esdt_safe_template().get();
        let metadata = self.blockchain().get_code_metadata(&source_address);

        self.tx()
            .typed(EnshrineEsdtSafeProxy)
            .init(
                is_sovereign_chain,
                token_handler_address,
                Some(wegld_identifier),
                Some(sov_token_prefix),
                opt_config,
            )
            .gas(60_000_000)
            .from_source(source_address)
            .code_metadata(metadata)
            .returns(ReturnsNewManagedAddress)
            .sync_call()
    }

    #[only_admin]
    #[endpoint(deployEsdtSafe)]
    fn deploy_mvx_esdt_safe(
        &self,
        opt_config: OptionalValue<EsdtSafeConfig<Self::Api>>,
    ) -> ManagedAddress {
        let source_address = self.mvx_esdt_safe_template().get();
        let metadata = self.blockchain().get_code_metadata(&source_address);

        self.tx()
            .typed(MvxEsdtSafeProxy)
            .init(opt_config)
            .gas(60_000_000)
            .from_source(source_address)
            .code_metadata(metadata)
            .returns(ReturnsNewManagedAddress)
            .sync_call()
    }

    #[only_admin]
    #[endpoint(deployFeeMarket)]
    fn deploy_fee_market(
        &self,
        esdt_safe_address: ManagedAddress,
        fee: Option<FeeStruct<Self::Api>>,
    ) -> ManagedAddress {
        let source_address = self.fee_market_template().get();
        let metadata = self.blockchain().get_code_metadata(&source_address);

        let fee_market_address = self
            .tx()
            .typed(FeeMarketProxy)
            .init(&esdt_safe_address, fee)
            .gas(60_000_000)
            .from_source(source_address)
            .code_metadata(metadata)
            .returns(ReturnsNewManagedAddress)
            .sync_call();

        self.tx()
            .to(&esdt_safe_address)
            .typed(MvxEsdtSafeProxy)
            .set_fee_market_address(&fee_market_address)
            .sync_call();

        fee_market_address
    }

    #[storage_mapper("chainConfigTemplate")]
    fn chain_config_template(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("headerVerifierTemplate")]
    fn header_verifier_template(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("crossChainOperationsTemplate")]
    fn mvx_esdt_safe_template(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("feeMarketTemplate")]
    fn fee_market_template(&self) -> SingleValueMapper<ManagedAddress>;
}
