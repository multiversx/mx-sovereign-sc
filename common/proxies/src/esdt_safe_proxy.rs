// Code generated by the multiversx-sc proxy generator. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![allow(dead_code)]
#![allow(clippy::all)]

use multiversx_sc::proxy_imports::*;

pub struct EsdtSafeProxy;

impl<Env, From, To, Gas> TxProxyTrait<Env, From, To, Gas> for EsdtSafeProxy
where
    Env: TxEnv,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    type TxProxyMethods = EsdtSafeProxyMethods<Env, From, To, Gas>;

    fn proxy_methods(self, tx: Tx<Env, From, To, (), Gas, (), ()>) -> Self::TxProxyMethods {
        EsdtSafeProxyMethods { wrapped_tx: tx }
    }
}

pub struct EsdtSafeProxyMethods<Env, From, To, Gas>
where
    Env: TxEnv,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    wrapped_tx: Tx<Env, From, To, (), Gas, (), ()>,
}

#[rustfmt::skip]
impl<Env, From, Gas> EsdtSafeProxyMethods<Env, From, (), Gas>
where
    Env: TxEnv,
    Env::Api: VMApi,
    From: TxFrom<Env>,
    Gas: TxGas<Env>,
{
    pub fn init<
        Arg0: ProxyArg<bool>,
    >(
        self,
        is_sovereign_chain: Arg0,
    ) -> TxTypedDeploy<Env, From, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_deploy()
            .argument(&is_sovereign_chain)
            .original_result()
    }
}

#[rustfmt::skip]
impl<Env, From, To, Gas> EsdtSafeProxyMethods<Env, From, To, Gas>
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
impl<Env, From, To, Gas> EsdtSafeProxyMethods<Env, From, To, Gas>
where
    Env: TxEnv,
    Env::Api: VMApi,
    From: TxFrom<Env>,
    To: TxTo<Env>,
    Gas: TxGas<Env>,
{
    pub fn set_fee_market_address<
        Arg0: ProxyArg<ManagedAddress<Env::Api>>,
    >(
        self,
        fee_market_address: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("setFeeMarketAddress")
            .argument(&fee_market_address)
            .original_result()
    }

    pub fn set_header_verifier_address<
        Arg0: ProxyArg<ManagedAddress<Env::Api>>,
    >(
        self,
        header_verifier_address: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("setHeaderVerifierAddress")
            .argument(&header_verifier_address)
            .original_result()
    }

    pub fn set_max_user_tx_gas_limit<
        Arg0: ProxyArg<u64>,
    >(
        self,
        max_user_tx_gas_limit: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("setMaxTxGasLimit")
            .argument(&max_user_tx_gas_limit)
            .original_result()
    }

    pub fn set_banned_endpoint<
        Arg0: ProxyArg<ManagedBuffer<Env::Api>>,
    >(
        self,
        endpoint_name: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("setBannedEndpoint")
            .argument(&endpoint_name)
            .original_result()
    }

    pub fn deposit<
        Arg0: ProxyArg<ManagedAddress<Env::Api>>,
        Arg1: ProxyArg<OptionalValue<MultiValue3<u64, ManagedBuffer<Env::Api>, ManagedVec<Env::Api, ManagedBuffer<Env::Api>>>>>,
    >(
        self,
        to: Arg0,
        optional_transfer_data: Arg1,
    ) -> TxTypedCall<Env, From, To, (), Gas, ()> {
        self.wrapped_tx
            .raw_call("deposit")
            .argument(&to)
            .argument(&optional_transfer_data)
            .original_result()
    }

    pub fn set_min_valid_signers<
        Arg0: ProxyArg<u32>,
    >(
        self,
        new_value: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("setMinValidSigners")
            .argument(&new_value)
            .original_result()
    }

    pub fn add_signers<
        Arg0: ProxyArg<MultiValueEncoded<Env::Api, ManagedAddress<Env::Api>>>,
    >(
        self,
        signers: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("addSigners")
            .argument(&signers)
            .original_result()
    }

    pub fn remove_signers<
        Arg0: ProxyArg<MultiValueEncoded<Env::Api, ManagedAddress<Env::Api>>>,
    >(
        self,
        signers: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("removeSigners")
            .argument(&signers)
            .original_result()
    }

    pub fn register_token<
        Arg0: ProxyArg<TokenIdentifier<Env::Api>>,
        Arg1: ProxyArg<EsdtTokenType>,
        Arg2: ProxyArg<ManagedBuffer<Env::Api>>,
        Arg3: ProxyArg<ManagedBuffer<Env::Api>>,
        Arg4: ProxyArg<usize>,
    >(
        self,
        sov_token_id: Arg0,
        token_type: Arg1,
        token_display_name: Arg2,
        token_ticker: Arg3,
        num_decimals: Arg4,
    ) -> TxTypedCall<Env, From, To, (), Gas, ()> {
        self.wrapped_tx
            .raw_call("registerToken")
            .argument(&sov_token_id)
            .argument(&token_type)
            .argument(&token_display_name)
            .argument(&token_ticker)
            .argument(&num_decimals)
            .original_result()
    }

    pub fn execute_operations<
        Arg0: ProxyArg<ManagedBuffer<Env::Api>>,
        Arg1: ProxyArg<operation::Operation<Env::Api>>,
    >(
        self,
        hash_of_hashes: Arg0,
        operation: Arg1,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("executeBridgeOps")
            .argument(&hash_of_hashes)
            .argument(&operation)
            .original_result()
    }

    pub fn set_max_bridged_amount<
        Arg0: ProxyArg<TokenIdentifier<Env::Api>>,
        Arg1: ProxyArg<BigUint<Env::Api>>,
    >(
        self,
        token_id: Arg0,
        max_amount: Arg1,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("setMaxBridgedAmount")
            .argument(&token_id)
            .argument(&max_amount)
            .original_result()
    }

    pub fn max_bridged_amount<
        Arg0: ProxyArg<TokenIdentifier<Env::Api>>,
    >(
        self,
        token_id: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, BigUint<Env::Api>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getMaxBridgedAmount")
            .argument(&token_id)
            .original_result()
    }

    /// Tokens in the whitelist can be transferred without fees 
    pub fn add_tokens_to_whitelist<
        Arg0: ProxyArg<MultiValueEncoded<Env::Api, TokenIdentifier<Env::Api>>>,
    >(
        self,
        tokens: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("addTokensToWhitelist")
            .argument(&tokens)
            .original_result()
    }

    pub fn remove_tokens_from_whitelist<
        Arg0: ProxyArg<MultiValueEncoded<Env::Api, TokenIdentifier<Env::Api>>>,
    >(
        self,
        tokens: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("removeTokensFromWhitelist")
            .argument(&tokens)
            .original_result()
    }

    /// Tokens in blacklist cannot be transferred 
    pub fn add_tokens_to_blacklist<
        Arg0: ProxyArg<MultiValueEncoded<Env::Api, TokenIdentifier<Env::Api>>>,
    >(
        self,
        tokens: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("addTokensToBlacklist")
            .argument(&tokens)
            .original_result()
    }

    pub fn remove_tokens_from_blacklist<
        Arg0: ProxyArg<MultiValueEncoded<Env::Api, TokenIdentifier<Env::Api>>>,
    >(
        self,
        tokens: Arg0,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("removeTokensFromBlacklist")
            .argument(&tokens)
            .original_result()
    }

    pub fn token_whitelist(
        self,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, MultiValueEncoded<Env::Api, TokenIdentifier<Env::Api>>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getTokenWhitelist")
            .original_result()
    }

    pub fn token_blacklist(
        self,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, MultiValueEncoded<Env::Api, TokenIdentifier<Env::Api>>> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("getTokenBlacklist")
            .original_result()
    }

    pub fn pause_endpoint(
        self,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("pause")
            .original_result()
    }

    pub fn unpause_endpoint(
        self,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("unpause")
            .original_result()
    }

    pub fn paused_status(
        self,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, bool> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("isPaused")
            .original_result()
    }
}
