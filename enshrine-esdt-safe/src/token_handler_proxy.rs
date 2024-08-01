// Code generated by the multiversx-sc proxy generator. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![allow(dead_code)]
#![allow(clippy::all)]

use multiversx_sc::proxy_imports::*;

pub struct TokenHandlerProxy;

impl<Env, From, To, Gas> TxProxyTrait<Env, From, To, Gas> for TokenHandlerProxy
where
    Env: TxEnv,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    type TxProxyMethods = TokenHandlerProxyMethods<Env, From, To, Gas>;

    fn proxy_methods(self, tx: Tx<Env, From, To, (), Gas, (), ()>) -> Self::TxProxyMethods {
        TokenHandlerProxyMethods { wrapped_tx: tx }
    }
}

pub struct TokenHandlerProxyMethods<Env, From, To, Gas>
where
    Env: TxEnv,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    wrapped_tx: Tx<Env, From, To, (), Gas, (), ()>,
}

#[rustfmt::skip]
impl<Env, From, Gas> TokenHandlerProxyMethods<Env, From, (), Gas>
where
    Env: TxEnv,
    Env::Api: VMApi,
    From: TxFrom<Env>,
    Gas: TxGas<Env>,
{
    pub fn init(
        self,
    ) -> TxTypedDeploy<Env, From, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_deploy()
            .original_result()
    }
}

#[rustfmt::skip]
impl<Env, From, To, Gas> TokenHandlerProxyMethods<Env, From, To, Gas>
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
impl<Env, From, To, Gas> TokenHandlerProxyMethods<Env, From, To, Gas>
where
    Env: TxEnv,
    Env::Api: VMApi,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    pub fn transfer_tokens<
        Arg0: ProxyArg<Option<transaction::TransferData<Env::Api>>>,
        Arg1: ProxyArg<ManagedAddress<Env::Api>>,
        Arg2: ProxyArg<MultiValueEncoded<Env::Api, transaction::OperationEsdtPayment<Env::Api>>>,
    >(
        self,
        opt_transfer_data: Arg0,
        to: Arg1,
        tokens: Arg2,
    ) -> TxTypedCall<Env, From, To, (), Gas, ()> {
        self.wrapped_tx
            .raw_call("transferTokens")
            .argument(&opt_transfer_data)
            .argument(&to)
            .argument(&tokens)
            .original_result()
    }

    pub fn burn_tokens_endpoint<
        Arg0: ProxyArg<transaction::Operation<Env::Api>>,
    >(
        self,
        operation: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("burnTokens")
            .argument(&operation)
            .original_result()
    }

    pub fn set_max_tx_batch_size<
        Arg0: ProxyArg<usize>,
    >(
        self,
        new_max_tx_batch_size: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("setMaxTxBatchSize")
            .argument(&new_max_tx_batch_size)
            .original_result()
    }

    pub fn set_max_tx_batch_block_duration<
        Arg0: ProxyArg<u64>,
    >(
        self,
        new_max_tx_batch_block_duration: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("setMaxTxBatchBlockDuration")
            .argument(&new_max_tx_batch_block_duration)
            .original_result()
    }

    pub fn get_current_tx_batch(
        self,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, OptionalValue<MultiValue2<u64, MultiValueEncoded<Env::Api, MultiValue7<u64, u64, ManagedAddress<Env::Api>, ManagedAddress<Env::Api>, ManagedVec<Env::Api, EsdtTokenPayment<Env::Api>>, ManagedVec<Env::Api, transaction::StolenFromFrameworkEsdtTokenData<Env::Api>>, Option<transaction::TransferData<Env::Api>>>>>>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getCurrentTxBatch")
            .original_result()
    }

    pub fn get_first_batch_any_status(
        self,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, OptionalValue<MultiValue2<u64, MultiValueEncoded<Env::Api, MultiValue7<u64, u64, ManagedAddress<Env::Api>, ManagedAddress<Env::Api>, ManagedVec<Env::Api, EsdtTokenPayment<Env::Api>>, ManagedVec<Env::Api, transaction::StolenFromFrameworkEsdtTokenData<Env::Api>>, Option<transaction::TransferData<Env::Api>>>>>>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getFirstBatchAnyStatus")
            .original_result()
    }

    pub fn get_batch<
        Arg0: ProxyArg<u64>,
    >(
        self,
        batch_id: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, OptionalValue<MultiValue2<u64, MultiValueEncoded<Env::Api, MultiValue7<u64, u64, ManagedAddress<Env::Api>, ManagedAddress<Env::Api>, ManagedVec<Env::Api, EsdtTokenPayment<Env::Api>>, ManagedVec<Env::Api, transaction::StolenFromFrameworkEsdtTokenData<Env::Api>>, Option<transaction::TransferData<Env::Api>>>>>>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getBatch")
            .argument(&batch_id)
            .original_result()
    }

    pub fn get_batch_status<
        Arg0: ProxyArg<u64>,
    >(
        self,
        batch_id: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, tx_batch_module::batch_status::BatchStatus<Env::Api>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getBatchStatus")
            .argument(&batch_id)
            .original_result()
    }

    pub fn first_batch_id(
        self,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, u64> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getFirstBatchId")
            .original_result()
    }

    pub fn last_batch_id(
        self,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, u64> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getLastBatchId")
            .original_result()
    }
}
