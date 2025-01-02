// Code generated by the multiversx-sc proxy generator. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![allow(dead_code)]
#![allow(clippy::all)]

use multiversx_sc::proxy_imports::*;

pub struct ChainFactoryContractProxy;

impl<Env, From, To, Gas> TxProxyTrait<Env, From, To, Gas> for ChainFactoryContractProxy
where
    Env: TxEnv,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    type TxProxyMethods = ChainFactoryContractProxyMethods<Env, From, To, Gas>;

    fn proxy_methods(self, tx: Tx<Env, From, To, (), Gas, (), ()>) -> Self::TxProxyMethods {
        ChainFactoryContractProxyMethods { wrapped_tx: tx }
    }
}

pub struct ChainFactoryContractProxyMethods<Env, From, To, Gas>
where
    Env: TxEnv,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    wrapped_tx: Tx<Env, From, To, (), Gas, (), ()>,
}

#[rustfmt::skip]
impl<Env, From, Gas> ChainFactoryContractProxyMethods<Env, From, (), Gas>
where
    Env: TxEnv,
    Env::Api: VMApi,
    From: TxFrom<Env>,
    Gas: TxGas<Env>,
{
    pub fn init<
        Arg0: ProxyArg<ManagedAddress<Env::Api>>,
        Arg1: ProxyArg<ManagedAddress<Env::Api>>,
        Arg2: ProxyArg<ManagedAddress<Env::Api>>,
        Arg3: ProxyArg<ManagedAddress<Env::Api>>,
        Arg4: ProxyArg<ManagedAddress<Env::Api>>,
    >(
        self,
        sovereign_forge_address: Arg0,
        chain_config_template: Arg1,
        header_verifier_template: Arg2,
        cross_chain_operation_template: Arg3,
        fee_market_template: Arg4,
    ) -> TxTypedDeploy<Env, From, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_deploy()
            .argument(&sovereign_forge_address)
            .argument(&chain_config_template)
            .argument(&header_verifier_template)
            .argument(&cross_chain_operation_template)
            .argument(&fee_market_template)
            .original_result()
    }
}

#[rustfmt::skip]
impl<Env, From, To, Gas> ChainFactoryContractProxyMethods<Env, From, To, Gas>
where
    Env: TxEnv,
    Env::Api: VMApi,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    pub fn upgrade(
        self,
    ) -> TxTypedUpgrade<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_upgrade()
            .original_result()
    }
}

#[rustfmt::skip]
impl<Env, From, To, Gas> ChainFactoryContractProxyMethods<Env, From, To, Gas>
where
    Env: TxEnv,
    Env::Api: VMApi,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    pub fn deploy_sovereign_chain_config_contract<
        Arg0: ProxyArg<u64>,
        Arg1: ProxyArg<u64>,
        Arg2: ProxyArg<BigUint<Env::Api>>,
        Arg3: ProxyArg<MultiValueEncoded<Env::Api, MultiValue2<TokenIdentifier<Env::Api>, BigUint<Env::Api>>>>,
    >(
        self,
        min_validators: Arg0,
        max_validators: Arg1,
        min_stake: Arg2,
        additional_stake_required: Arg3,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ManagedAddress<Env::Api>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("deploySovereignChainConfigContract")
            .argument(&min_validators)
            .argument(&max_validators)
            .argument(&min_stake)
            .argument(&additional_stake_required)
            .original_result()
    }

    pub fn deploy_header_verifier<
        Arg0: ProxyArg<MultiValueEncoded<Env::Api, ManagedBuffer<Env::Api>>>,
    >(
        self,
        bls_pub_keys: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ManagedAddress<Env::Api>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("deployHeaderVerifier")
            .argument(&bls_pub_keys)
            .original_result()
    }

    pub fn deploy_enshrine_esdt_safe<
        Arg0: ProxyArg<bool>,
        Arg1: ProxyArg<ManagedAddress<Env::Api>>,
        Arg2: ProxyArg<TokenIdentifier<Env::Api>>,
        Arg3: ProxyArg<ManagedBuffer<Env::Api>>,
        Arg4: ProxyArg<transaction::BridgeConfig<Env::Api>>,
    >(
        self,
        is_sovereign_chain: Arg0,
        token_handler_address: Arg1,
        wegld_identifier: Arg2,
        sov_token_prefix: Arg3,
        config: Arg4,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ManagedAddress<Env::Api>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("deployEnshrineEsdtSafe")
            .argument(&is_sovereign_chain)
            .argument(&token_handler_address)
            .argument(&wegld_identifier)
            .argument(&sov_token_prefix)
            .argument(&config)
            .original_result()
    }

    pub fn deploy_esdt_safe<
        Arg0: ProxyArg<bool>,
        Arg1: ProxyArg<ManagedAddress<Env::Api>>,
    >(
        self,
        is_sovereign_chain: Arg0,
        header_verifier_address: Arg1,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ManagedAddress<Env::Api>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("deployEsdtSafe")
            .argument(&is_sovereign_chain)
            .argument(&header_verifier_address)
            .original_result()
    }

    pub fn deploy_fee_market<
        Arg0: ProxyArg<ManagedAddress<Env::Api>>,
        Arg1: ProxyArg<Option<super::fee_market_proxy::FeeStruct<Env::Api>>>,
    >(
        self,
        esdt_safe_address: Arg0,
        fee: Arg1,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ManagedAddress<Env::Api>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("deployFeeMarket")
            .argument(&esdt_safe_address)
            .argument(&fee)
            .original_result()
    }

    pub fn complete_setup_phase<
        Arg0: ProxyArg<ManagedAddress<Env::Api>>,
    >(
        self,
        _contract_address: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("completeSetupPhase")
            .argument(&_contract_address)
            .original_result()
    }

    pub fn is_admin<
        Arg0: ProxyArg<ManagedAddress<Env::Api>>,
    >(
        self,
        address: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, bool> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("isAdmin")
            .argument(&address)
            .original_result()
    }

    pub fn add_admin<
        Arg0: ProxyArg<ManagedAddress<Env::Api>>,
    >(
        self,
        address: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("addAdmin")
            .argument(&address)
            .original_result()
    }

    pub fn remove_admin<
        Arg0: ProxyArg<ManagedAddress<Env::Api>>,
    >(
        self,
        address: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("removeAdmin")
            .argument(&address)
            .original_result()
    }

    pub fn admins(
        self,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, MultiValueEncoded<Env::Api, ManagedAddress<Env::Api>>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getAdmins")
            .original_result()
    }
}
