use crate::err_msg;
use multiversx_sc::types::{MultiValueEncoded, ReturnsResult};
use proxies::chain_factory_proxy::ChainFactoryContractProxy;
use transaction::StakeMultiArg;

#[multiversx_sc::module]
pub trait ScDeployModule: super::utils::UtilsModule + super::storage::StorageModule {
    fn deploy_chain_config(
        &self,
        min_validators: u64,
        max_validators: u64,
        min_stake: BigUint,
        additional_stake_required: MultiValueEncoded<StakeMultiArg<Self::Api>>,
    ) -> ManagedAddress {
        let chain_factory_address = self.get_caller_shard_id();

        self.tx()
            .to(chain_factory_address)
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

    fn deploy_header_verifier(&self, bls_keys: MultiValueEncoded<ManagedBuffer>) -> ManagedAddress {
        let chain_factory_address = self.get_caller_shard_id();

        self.tx()
            .to(chain_factory_address)
            .typed(ChainFactoryContractProxy)
            .deploy_header_verifier(bls_keys)
            .returns(ReturnsResult)
            .sync_call()
    }

    fn deploy_esdt_safe(&self, is_sovereign_chain: bool) -> ManagedAddress {
        let chain_factory_address = self.get_caller_shard_id();

        self.tx()
            .to(chain_factory_address)
            .typed(ChainFactoryContractProxy)
            .deploy_esdt_safe(is_sovereign_chain)
            .returns(ReturnsResult)
            .sync_call()
    }
}
