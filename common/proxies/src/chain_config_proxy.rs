// Code generated by the multiversx-sc proxy generator. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![allow(dead_code)]
#![allow(clippy::all)]

use multiversx_sc::proxy_imports::*;

pub struct ChainConfigContractProxy;

impl<Env, From, To, Gas> TxProxyTrait<Env, From, To, Gas> for ChainConfigContractProxy
where
    Env: TxEnv,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    type TxProxyMethods = ChainConfigContractProxyMethods<Env, From, To, Gas>;

    fn proxy_methods(self, tx: Tx<Env, From, To, (), Gas, (), ()>) -> Self::TxProxyMethods {
        ChainConfigContractProxyMethods { wrapped_tx: tx }
    }
}

pub struct ChainConfigContractProxyMethods<Env, From, To, Gas>
where
    Env: TxEnv,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    wrapped_tx: Tx<Env, From, To, (), Gas, (), ()>,
}

#[rustfmt::skip]
impl<Env, From, Gas> ChainConfigContractProxyMethods<Env, From, (), Gas>
where
    Env: TxEnv,
    Env::Api: VMApi,
    From: TxFrom<Env>,
    Gas: TxGas<Env>,
{
    pub fn init<
        Arg0: ProxyArg<transaction::SovereignConfig<Env::Api>>,
        Arg1: ProxyArg<ManagedAddress<Env::Api>>,
    >(
        self,
        config: Arg0,
        admin: Arg1,
    ) -> TxTypedDeploy<Env, From, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_deploy()
            .argument(&config)
            .argument(&admin)
            .original_result()
    }
}

#[rustfmt::skip]
impl<Env, From, To, Gas> ChainConfigContractProxyMethods<Env, From, To, Gas>
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
impl<Env, From, To, Gas> ChainConfigContractProxyMethods<Env, From, To, Gas>
where
    Env: TxEnv,
    Env::Api: VMApi,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    pub fn update_config<
        Arg0: ProxyArg<transaction::SovereignConfig<Env::Api>>,
    >(
        self,
        new_config: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("updateConfig")
            .argument(&new_config)
            .original_result()
    }

    pub fn complete_setup_phase<
        Arg0: ProxyArg<ManagedAddress<Env::Api>>,
    >(
        self,
        header_verifier_address: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("completeSetupPhase")
            .argument(&header_verifier_address)
            .original_result()
    }

    pub fn sovereign_config(
        self,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, transaction::SovereignConfig<Env::Api>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("sovereignConfig")
            .original_result()
    }

    pub fn was_previously_slashed<
        Arg0: ProxyArg<ManagedAddress<Env::Api>>,
    >(
        self,
        validator: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, bool> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("wasPreviouslySlashed")
            .argument(&validator)
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
