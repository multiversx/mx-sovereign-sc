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
        Arg5: ProxyArg<ManagedAddress<Env::Api>>,
        Arg6: ProxyArg<BigUint<Env::Api>>,
    >(
        self,
        validators_contract_address: Arg0,
        chain_config_template: Arg1,
        header_verifier_template: Arg2,
        cross_chain_operation_template: Arg3,
        fee_market_template: Arg4,
        token_handler_template: Arg5,
        deploy_cost: Arg6,
    ) -> TxTypedDeploy<Env, From, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_deploy()
            .argument(&validators_contract_address)
            .argument(&chain_config_template)
            .argument(&header_verifier_template)
            .argument(&cross_chain_operation_template)
            .argument(&fee_market_template)
            .argument(&token_handler_template)
            .argument(&deploy_cost)
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
        Arg0: ProxyArg<usize>,
        Arg1: ProxyArg<usize>,
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
            .raw_call("deploySovereignChainConfigContract")
            .argument(&min_validators)
            .argument(&max_validators)
            .argument(&min_stake)
            .argument(&additional_stake_required)
            .original_result()
    }

    pub fn deploy_header_verifier<
        Arg0: ProxyArg<ManagedBuffer<Env::Api>>,
        Arg1: ProxyArg<MultiValueEncoded<Env::Api, ManagedBuffer<Env::Api>>>,
    >(
        self,
        chain_id: Arg0,
        bls_pub_keys: Arg1,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("deployHeaderVerifier")
            .argument(&chain_id)
            .argument(&bls_pub_keys)
            .original_result()
    }

    pub fn deploy_cross_chain_operation<
        Arg0: ProxyArg<ManagedBuffer<Env::Api>>,
        Arg1: ProxyArg<bool>,
        Arg2: ProxyArg<Option<TokenIdentifier<Env::Api>>>,
        Arg3: ProxyArg<Option<ManagedBuffer<Env::Api>>>,
    >(
        self,
        chain_id: Arg0,
        is_sovereign_chain: Arg1,
        opt_wegld_identifier: Arg2,
        opt_sov_token_prefix: Arg3,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("deployCrossChainOperation")
            .argument(&chain_id)
            .argument(&is_sovereign_chain)
            .argument(&opt_wegld_identifier)
            .argument(&opt_sov_token_prefix)
            .original_result()
    }

    pub fn deploy_fee_market<
        Arg0: ProxyArg<ManagedBuffer<Env::Api>>,
        Arg1: ProxyArg<ManagedAddress<Env::Api>>,
        Arg2: ProxyArg<ManagedAddress<Env::Api>>,
    >(
        self,
        chain_id: Arg0,
        esdt_safe_address: Arg1,
        price_aggregator_address: Arg2,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("deployFeeMarket")
            .argument(&chain_id)
            .argument(&esdt_safe_address)
            .argument(&price_aggregator_address)
            .original_result()
    }

    pub fn add_contracts_to_map<
        Arg0: ProxyArg<ManagedBuffer<Env::Api>>,
        Arg1: ProxyArg<MultiValueEncoded<Env::Api, ContractInfo<Env::Api>>>,
    >(
        self,
        chain_id: Arg0,
        contracts_info: Arg1,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("addContractsToMap")
            .argument(&chain_id)
            .argument(&contracts_info)
            .original_result()
    }

    pub fn slash<
        Arg0: ProxyArg<ManagedBuffer<Env::Api>>,
        Arg1: ProxyArg<ManagedAddress<Env::Api>>,
        Arg2: ProxyArg<BigUint<Env::Api>>,
    >(
        self,
        _chain_id: Arg0,
        validator_address: Arg1,
        value: Arg2,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("slash")
            .argument(&_chain_id)
            .argument(&validator_address)
            .argument(&value)
            .original_result()
    }

    pub fn distribute_slashed<
        Arg0: ProxyArg<ManagedBuffer<Env::Api>>,
        Arg1: ProxyArg<MultiValueEncoded<Env::Api, MultiValue2<ManagedAddress<Env::Api>, BigUint<Env::Api>>>>,
    >(
        self,
        _chain_id: Arg0,
        dest_amount_pairs: Arg1,
    ) -> TxTypedCall<Env, From, To, NotPayable, Gas, ()> {
        self.wrapped_tx
            .payment(NotPayable)
            .raw_call("distributeSlashed")
            .argument(&_chain_id)
            .argument(&dest_amount_pairs)
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

#[type_abi]
#[derive(TopEncode, TopDecode)]
pub struct ContractInfo<Api>
where
    Api: ManagedTypeApi,
{
    pub id: ScArray,
    pub address: ManagedAddress<Api>,
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, ManagedVecItem, PartialEq)]
pub enum ScArray {
    ChainFactory,
    Controller,
    SovereignHeaderVerifier,
    SovereignCrossChainOperation,
    FeeMarket,
    TokenHandler,
    ChainConfig,
    Slashing,
}
