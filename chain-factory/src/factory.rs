use multiversx_sc::imports::*;
use multiversx_sc_modules::only_admin;
use proxies::{
    chain_config_proxy::ChainConfigContractProxy,
    enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy,
    esdt_safe_proxy::EsdtSafeProxy,
    fee_market_proxy::{FeeMarketProxy, FeeStruct},
    header_verifier_proxy::HeaderverifierProxy,
};
use transaction::StakeMultiArg;
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait FactoryModule: only_admin::OnlyAdminModule {
    #[only_admin]
    #[endpoint(deploySovereignChainConfigContract)]
    fn deploy_sovereign_chain_config_contract(
        &self,
        min_validators: u64,
        max_validators: u64,
        min_stake: BigUint,
        additional_stake_required: MultiValueEncoded<StakeMultiArg<Self::Api>>,
    ) -> ManagedAddress {
        let caller = self.blockchain().get_caller();
        let source_address = self.chain_config_template().get();
        let metadata = self.blockchain().get_code_metadata(&source_address);

        self.tx()
            .typed(ChainConfigContractProxy)
            .init(
                min_validators,
                max_validators,
                min_stake,
                &caller,
                additional_stake_required,
            )
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
        bls_pub_keys: MultiValueEncoded<ManagedBuffer>,
    ) -> ManagedAddress {
        let source_address = self.header_verifier_template().get();
        let metadata = self.blockchain().get_code_metadata(&source_address);

        self.tx()
            .typed(HeaderverifierProxy)
            .init(bls_pub_keys)
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
    ) -> ManagedAddress {
        let source_address = self.enshrine_esdt_safe_template().get();
        let metadata = self.blockchain().get_code_metadata(&source_address);

        self.tx()
            .typed(EnshrineEsdtSafeProxy)
            .init(
                is_sovereign_chain,
                token_handler_address,
                Some(wegld_identifier),
                Some(sov_token_prefix),
            )
            .gas(60_000_000)
            .from_source(source_address)
            .code_metadata(metadata)
            .returns(ReturnsNewManagedAddress)
            .sync_call()
    }

    #[only_admin]
    #[endpoint(deployEsdtSafe)]
    fn deploy_esdt_safe(
        &self,
        is_sovereign_chain: bool,
        header_verifier_address: ManagedAddress,
    ) -> ManagedAddress {
        let source_address = self.enshrine_esdt_safe_template().get();
        let metadata = self.blockchain().get_code_metadata(&source_address);

        let esdt_safe_address = self
            .tx()
            .typed(EsdtSafeProxy)
            .init(is_sovereign_chain)
            .gas(60_000_000)
            .from_source(source_address)
            .code_metadata(metadata)
            .returns(ReturnsNewManagedAddress)
            .sync_call();

        self.tx()
            .to(header_verifier_address)
            .typed(HeaderverifierProxy)
            .set_esdt_safe_address(&esdt_safe_address)
            .sync_call();

        esdt_safe_address
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

        self.tx()
            .typed(FeeMarketProxy)
            .init(esdt_safe_address, fee)
            .gas(60_000_000)
            .from_source(source_address)
            .code_metadata(metadata)
            .returns(ReturnsNewManagedAddress)
            .sync_call()
    }

    #[only_admin]
    #[endpoint(completeSetupPhase)]
    fn complete_setup_phase(&self, _contract_address: ManagedAddress) {
        // TODO: will have to call each contract's endpoint to finish setup phase
    }

    #[storage_mapper("chainConfigTemplate")]
    fn chain_config_template(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("headerVerifierTemplate")]
    fn header_verifier_template(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("crossChainOperationsTemplate")]
    fn enshrine_esdt_safe_template(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("feeMarketTemplate")]
    fn fee_market_template(&self) -> SingleValueMapper<ManagedAddress>;
}
