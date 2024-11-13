use chain_config::StakeMultiArg;

use multiversx_sc::imports::*;
use multiversx_sc_modules::only_admin;
use proxies::{
    chain_config_proxy::ChainConfigContractProxy,
    enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy,
    fee_market_proxy::{FeeMarketProxy, FeeStruct},
    header_verifier_proxy::HeaderverifierProxy,
};
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait FactoryModule:
    only_admin::OnlyAdminModule + crate::common::storage::CommonStorage
{
    // TODO: Check if contract was already deployed
    #[payable("EGLD")]
    #[endpoint(deploySovereignChainConfigContract)]
    fn deploy_sovereign_chain_config_contract(
        &self,
        min_validators: usize,
        max_validators: usize,
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
    #[endpoint(deployCrossChainOperation)]
    fn deploy_cross_chain_operation(
        &self,
        is_sovereign_chain: bool,
        opt_wegld_identifier: Option<TokenIdentifier>,
        opt_sov_token_prefix: Option<ManagedBuffer>,
    ) -> ManagedAddress {
        let source_address = self.cross_chain_operations_template().get();
        let token_handler_address = self.token_handler_template().get();
        let metadata = self.blockchain().get_code_metadata(&source_address);

        self.tx()
            .typed(EnshrineEsdtSafeProxy)
            .init(
                is_sovereign_chain,
                token_handler_address,
                opt_wegld_identifier,
                opt_sov_token_prefix,
            )
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

        self.tx()
            .typed(FeeMarketProxy)
            .init(esdt_safe_address, fee)
            .gas(60_000_000)
            .from_source(source_address)
            .code_metadata(metadata)
            .returns(ReturnsNewManagedAddress)
            .sync_call()
    }
}
