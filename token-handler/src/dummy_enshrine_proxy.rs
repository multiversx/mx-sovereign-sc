// Code generated by the multiversx-sc proxy generator. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![allow(dead_code)]
#![allow(clippy::all)]

use multiversx_sc::proxy_imports::*;

pub struct PairMockProxy;

impl<Env, From, To, Gas> TxProxyTrait<Env, From, To, Gas> for PairMockProxy
where
    Env: TxEnv,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    type TxProxyMethods = PairMockProxyMethods<Env, From, To, Gas>;

    fn proxy_methods(self, tx: Tx<Env, From, To, (), Gas, (), ()>) -> Self::TxProxyMethods {
        PairMockProxyMethods { wrapped_tx: tx }
    }
}

pub struct PairMockProxyMethods<Env, From, To, Gas>
where
    Env: TxEnv,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    wrapped_tx: Tx<Env, From, To, (), Gas, (), ()>,
}

#[rustfmt::skip]
impl<Env, From, Gas> PairMockProxyMethods<Env, From, (), Gas>
where
    Env: TxEnv,
    Env::Api: VMApi,
    From: TxFrom<Env>,
    Gas: TxGas<Env>,
{
    pub fn init<
        Arg0: ProxyArg<TokenIdentifier<Env::Api>>,
    >(
        self,
        usdc_token_id: Arg0,
    ) -> TxTypedDeploy<Env, From, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_deploy()
            .argument(&usdc_token_id)
            .original_result()
    }
}

#[rustfmt::skip]
impl<Env, From, To, Gas> PairMockProxyMethods<Env, From, To, Gas>
where
    Env: TxEnv,
    Env::Api: VMApi,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    pub fn get_safe_price_by_timestamp_offset<
        Arg0: ProxyArg<ManagedAddress<Env::Api>>,
        Arg1: ProxyArg<u64>,
        Arg2: ProxyArg<EsdtTokenPayment<Env::Api>>,
    >(
        self,
        _pair_address: Arg0,
        _timestamp_offset: Arg1,
        input_payment: Arg2,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, EsdtTokenPayment<Env::Api>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getSafePriceByTimestampOffset")
            .argument(&_pair_address)
            .argument(&_timestamp_offset)
            .argument(&input_payment)
            .original_result()
    }
}
