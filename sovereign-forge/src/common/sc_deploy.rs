use crate::err_msg;
use multiversx_sc::types::{MultiValueEncoded, ReturnsResult};
use proxies::{chain_factory_proxy::ChainFactoryContractProxy, fee_market_proxy::FeeStruct};
use transaction::aliases::StakeMultiArg;

#[multiversx_sc::module]
pub trait ScDeployModule: super::utils::UtilsModule + super::storage::StorageModule {
    #[inline]
    fn deploy_chain_config(
        &self,
        min_validators: u64,
        max_validators: u64,
        min_stake: BigUint,
        additional_stake_required: MultiValueEncoded<StakeMultiArg<Self::Api>>,
    ) -> ManagedAddress {
        self.tx()
            .to(self.get_chain_factory_address())
            .typed(ChainFactoryContractProxy)
            .deploy_sovereign_chain_config_contract(
                min_validators,
                max_validators,
                min_stake,
                additional_stake_required,
            )
            .returns(ReturnsResult)
            .sync_call()
    }

    #[inline]
    fn deploy_header_verifier(&self, bls_keys: MultiValueEncoded<ManagedBuffer>) -> ManagedAddress {
        self.tx()
            .to(self.get_chain_factory_address())
            .typed(ChainFactoryContractProxy)
            .deploy_header_verifier(bls_keys)
            .returns(ReturnsResult)
            .sync_call()
    }

    #[inline]
    fn deploy_esdt_safe(
        &self,
        is_sovereign_chain: bool,
        header_verifier_address: &ManagedAddress,
    ) -> ManagedAddress {
        self.tx()
            .to(self.get_chain_factory_address())
            .typed(ChainFactoryContractProxy)
            .deploy_esdt_safe(is_sovereign_chain, header_verifier_address)
            .returns(ReturnsResult)
            .sync_call()
    }

    #[inline]
    fn deploy_fee_market(
        &self,
        esdt_safe_address: &ManagedAddress,
        fee: Option<FeeStruct<Self::Api>>,
    ) -> ManagedAddress {
        self.tx()
            .to(self.get_chain_factory_address())
            .typed(ChainFactoryContractProxy)
            .deploy_fee_market(esdt_safe_address, fee)
            .returns(ReturnsResult)
            .sync_call()
    }
}
