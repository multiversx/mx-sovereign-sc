use multiversx_sc::api::CryptoApi;

use crate::{
    aliases::{GasLimit, TxNonce},
    generate_hash::GenerateHash,
    DEFAULT_MAX_TX_GAS_LIMIT,
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[type_abi]
#[derive(
    TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem, Clone, Debug, PartialEq,
)]
pub struct UpdateRegistrationStatusOperation {
    pub registration_status: u8,
    pub nonce: TxNonce,
}

#[type_abi]
#[derive(
    TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem, Clone, Debug, PartialEq,
)]
pub struct SetBurnMechanismOperation<M: ManagedTypeApi> {
    pub token_id: EgldOrEsdtTokenIdentifier<M>,
    pub nonce: TxNonce,
}

impl<A: CryptoApi> GenerateHash<A> for SetBurnMechanismOperation<A> {}

#[type_abi]
#[derive(
    TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem, Clone, Debug, PartialEq,
)]
pub struct SetLockMechanismOperation<M: ManagedTypeApi> {
    pub token_id: EgldOrEsdtTokenIdentifier<M>,
    pub nonce: TxNonce,
}

impl<A: CryptoApi> GenerateHash<A> for SetLockMechanismOperation<A> {}

#[type_abi]
#[derive(
    TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem, Clone, Debug, PartialEq,
)]
pub struct UpdateSovereignConfigOperation<M: ManagedTypeApi> {
    pub sovereign_config: SovereignConfig<M>,
    pub nonce: TxNonce,
}

impl<A: CryptoApi> GenerateHash<A> for UpdateSovereignConfigOperation<A> {}

#[type_abi]
#[derive(
    TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem, Clone, Debug, PartialEq,
)]
pub struct SovereignConfig<M: ManagedTypeApi> {
    pub min_validators: u64,
    pub max_validators: u64,
    pub min_stake: BigUint<M>,
    pub opt_additional_stake_required: Option<ManagedVec<M, StakeArgs<M>>>,
}

impl<A: CryptoApi> GenerateHash<A> for SovereignConfig<A> {}

impl<M: ManagedTypeApi> SovereignConfig<M> {
    pub fn new(
        min_validators: u64,
        max_validators: u64,
        min_stake: BigUint<M>,
        opt_additional_stake_required: Option<ManagedVec<M, StakeArgs<M>>>,
    ) -> Self {
        SovereignConfig {
            min_validators,
            max_validators,
            min_stake,
            opt_additional_stake_required,
        }
    }

    pub fn default_config() -> Self {
        SovereignConfig::new(2, 50, BigUint::default(), None)
    }

    pub fn default_config_for_test() -> Self {
        SovereignConfig::new(1, 2, BigUint::default(), None)
    }
}

#[type_abi]
#[derive(
    TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem, Clone, Debug, PartialEq,
)]
pub struct StakeArgs<M: ManagedTypeApi> {
    pub token_identifier: TokenIdentifier<M>,
    pub amount: BigUint<M>,
}

impl<M: ManagedTypeApi> StakeArgs<M> {
    pub fn new(token_identifier: TokenIdentifier<M>, amount: BigUint<M>) -> Self {
        StakeArgs {
            token_identifier,
            amount,
        }
    }
}

#[type_abi]
#[derive(
    TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem, Clone, Debug, PartialEq,
)]
pub struct MaxBridgedAmount<M: ManagedTypeApi> {
    pub token_id: EgldOrEsdtTokenIdentifier<M>,
    pub amount: BigUint<M>,
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem, Clone)]
pub struct UpdateEsdtSafeConfigOperation<M: ManagedTypeApi> {
    pub esdt_safe_config: EsdtSafeConfig<M>,
    pub nonce: TxNonce,
}

impl<A: CryptoApi> GenerateHash<A> for UpdateEsdtSafeConfigOperation<A> {}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem, Clone)]
pub struct EsdtSafeConfig<M: ManagedTypeApi> {
    pub token_whitelist: ManagedVec<M, EgldOrEsdtTokenIdentifier<M>>,
    pub token_blacklist: ManagedVec<M, EgldOrEsdtTokenIdentifier<M>>,
    pub max_tx_gas_limit: GasLimit,
    pub banned_endpoints: ManagedVec<M, ManagedBuffer<M>>,
    pub address_blacklist: ManagedVec<M, ManagedAddress<M>>,
    pub max_bridged_token_amounts: ManagedVec<M, MaxBridgedAmount<M>>,
}

impl<A: CryptoApi> GenerateHash<A> for EsdtSafeConfig<A> {}

impl<M: ManagedTypeApi> EsdtSafeConfig<M> {
    #[inline]
    pub fn default_config() -> Self {
        EsdtSafeConfig {
            token_whitelist: ManagedVec::new(),
            token_blacklist: ManagedVec::new(),
            max_tx_gas_limit: DEFAULT_MAX_TX_GAS_LIMIT,
            banned_endpoints: ManagedVec::new(),
            address_blacklist: ManagedVec::new(),
            max_bridged_token_amounts: ManagedVec::new(),
        }
    }

    pub fn new(
        token_whitelist: ManagedVec<M, EgldOrEsdtTokenIdentifier<M>>,
        token_blacklist: ManagedVec<M, EgldOrEsdtTokenIdentifier<M>>,
        max_tx_gas_limit: GasLimit,
        banned_endpoints: ManagedVec<M, ManagedBuffer<M>>,
        deposit_blacklist: ManagedVec<M, ManagedAddress<M>>,
        max_bridged_token_amounts: ManagedVec<M, MaxBridgedAmount<M>>,
    ) -> Self {
        EsdtSafeConfig {
            token_whitelist,
            token_blacklist,
            max_tx_gas_limit,
            banned_endpoints,
            address_blacklist: deposit_blacklist,
            max_bridged_token_amounts,
        }
    }
}
