// Code generated by the multiversx-sc proxy generator. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![allow(dead_code)]
#![allow(clippy::all)]

use multiversx_sc::proxy_imports::*;

pub struct SovereignForgeProxy;

impl<Env, From, To, Gas> TxProxyTrait<Env, From, To, Gas> for SovereignForgeProxy
where
    Env: TxEnv,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    type TxProxyMethods = SovereignForgeProxyMethods<Env, From, To, Gas>;

    fn proxy_methods(self, tx: Tx<Env, From, To, (), Gas, (), ()>) -> Self::TxProxyMethods {
        SovereignForgeProxyMethods { wrapped_tx: tx }
    }
}

pub struct SovereignForgeProxyMethods<Env, From, To, Gas>
where
    Env: TxEnv,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    wrapped_tx: Tx<Env, From, To, (), Gas, (), ()>,
}

#[rustfmt::skip]
impl<Env, From, Gas> SovereignForgeProxyMethods<Env, From, (), Gas>
where
    Env: TxEnv,
    Env::Api: VMApi,
    From: TxFrom<Env>,
    Gas: TxGas<Env>,
{
    pub fn init<
        Arg0: ProxyArg<BigUint<Env::Api>>,
    >(
        self,
        deploy_cost: Arg0,
    ) -> TxTypedDeploy<Env, From, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_deploy()
            .argument(&deploy_cost)
            .original_result()
    }
}

#[rustfmt::skip]
impl<Env, From, To, Gas> SovereignForgeProxyMethods<Env, From, To, Gas>
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
impl<Env, From, To, Gas> SovereignForgeProxyMethods<Env, From, To, Gas>
where
    Env: TxEnv,
    Env::Api: VMApi,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    pub fn register_token_handler<
        Arg0: ProxyArg<u32>,
        Arg1: ProxyArg<ManagedAddress<Env::Api>>,
    >(
        self,
        shard_id: Arg0,
        token_handler_address: Arg1,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("registerTokenHandler")
            .argument(&shard_id)
            .argument(&token_handler_address)
            .original_result()
    }

    pub fn register_chain_factory<
        Arg0: ProxyArg<u32>,
        Arg1: ProxyArg<ManagedAddress<Env::Api>>,
    >(
        self,
        shard_id: Arg0,
        chain_factory_address: Arg1,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("registerChainFactory")
            .argument(&shard_id)
            .argument(&chain_factory_address)
            .original_result()
    }

    pub fn complete_setup_phase(
        self,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("completeSetupPhase")
            .original_result()
    }

    pub fn deploy_phase_one<
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
    ) -> TxTypedCall<Env, From, To, (), Gas, ()> {
        self.wrapped_tx
            .raw_call("deployPhaseOne")
            .argument(&min_validators)
            .argument(&max_validators)
            .argument(&min_stake)
            .argument(&additional_stake_required)
            .original_result()
    }

    pub fn deploy_phase_two<
        Arg0: ProxyArg<MultiValueEncoded<Env::Api, ManagedBuffer<Env::Api>>>,
    >(
        self,
        bls_keys: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("deployPhaseTwo")
            .argument(&bls_keys)
            .original_result()
    }

    pub fn deploy_phase_three<
        Arg0: ProxyArg<bool>,
        Arg1: ProxyArg<ManagedAddress<Env::Api>>,
    >(
        self,
        is_sovereign_chain: Arg0,
        header_verifier_address: Arg1,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("deployPhaseThree")
            .argument(&is_sovereign_chain)
            .argument(&header_verifier_address)
            .original_result()
    }

    pub fn set_address<
        Arg0: ProxyArg<ManagedAddress<Env::Api>>,
        Arg1: ProxyArg<ManagedAddress<Env::Api>>,
    >(
        self,
        esdt_safe_address: Arg0,
        header_verifier_address: Arg1,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("setAddress")
            .argument(&esdt_safe_address)
            .argument(&header_verifier_address)
            .original_result()
    }

    pub fn chain_factories<
        Arg0: ProxyArg<u32>,
    >(
        self,
        shard_id: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ManagedAddress<Env::Api>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getChainFactoryAddress")
            .argument(&shard_id)
            .original_result()
    }

    pub fn token_handlers<
        Arg0: ProxyArg<u32>,
    >(
        self,
        shard_id: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ManagedAddress<Env::Api>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getTokenHandlerAddress")
            .argument(&shard_id)
            .original_result()
    }

    pub fn deploy_cost(
        self,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, BigUint<Env::Api>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getDeployCost")
            .original_result()
    }

    pub fn chain_ids(
        self,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, MultiValueEncoded<Env::Api, ManagedBuffer<Env::Api>>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getAllChainIds")
            .original_result()
    }
}
