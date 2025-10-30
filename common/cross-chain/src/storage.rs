use structs::{aliases::TxNonce, configs::EsdtSafeConfig, fee::FeeType, EsdtInfo};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait CrossChainStorage {
    #[storage_mapper("lastTxNonce")]
    fn last_tx_nonce(&self) -> SingleValueMapper<TxNonce>;

    #[storage_mapper("sovTokenPrefix")]
    fn sov_token_prefix(&self) -> SingleValueMapper<ManagedBuffer<Self::Api>>;

    #[storage_mapper("crossChainConfig")]
    fn esdt_safe_config(&self) -> SingleValueMapper<EsdtSafeConfig<Self::Api>>;

    #[storage_mapper("feeMarketAddress")]
    fn fee_market_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getSovToMvxTokenId)]
    #[storage_mapper("sovToMvxTokenId")]
    fn sovereign_to_multiversx_token_id_mapper(
        &self,
        sov_token_id: &EgldOrEsdtTokenIdentifier<Self::Api>,
    ) -> SingleValueMapper<EgldOrEsdtTokenIdentifier<Self::Api>>;

    #[view(getMvxToSovTokenId)]
    #[storage_mapper("mvxToSovTokenId")]
    fn multiversx_to_sovereign_token_id_mapper(
        &self,
        mvx_token_id: &EgldOrEsdtTokenIdentifier<Self::Api>,
    ) -> SingleValueMapper<EgldOrEsdtTokenIdentifier<Self::Api>>;

    #[view(getSovEsdtTokenInfo)]
    #[storage_mapper("sovEsdtTokenInfoMapper")]
    fn sovereign_to_multiversx_esdt_info_mapper(
        &self,
        token_identifier: &EgldOrEsdtTokenIdentifier<Self::Api>,
        nonce: u64,
    ) -> SingleValueMapper<EsdtInfo<Self::Api>>;

    #[view(getMvxEsdtTokenInfo)]
    #[storage_mapper("mvxEsdtTokenInfoMapper")]
    fn multiversx_to_sovereign_esdt_info_mapper(
        &self,
        token_identifier: &EgldOrEsdtTokenIdentifier<Self::Api>,
        nonce: u64,
    ) -> SingleValueMapper<EsdtInfo<Self::Api>>;

    #[view(getNativeToken)]
    #[storage_mapper("nativeToken")]
    fn native_token(&self) -> SingleValueMapper<EgldOrEsdtTokenIdentifier<Self::Api>>;

    #[storage_mapper("isSovereignChain")]
    fn is_sovereign_chain(&self) -> SingleValueMapper<bool>;

    #[storage_mapper_from_address("feeEnabledFlag")]
    fn external_fee_enabled(
        &self,
        sc_address: ManagedAddress,
    ) -> SingleValueMapper<bool, ManagedAddress>;

    #[storage_mapper_from_address("tokenFee")]
    fn external_token_fee(
        &self,
        sc_address: ManagedAddress,
        token_id: &TokenIdentifier,
    ) -> SingleValueMapper<FeeType<Self::Api>, ManagedAddress>;

    #[view(getDepositCallersBlacklist)]
    #[storage_mapper("depositCallersBlacklist")]
    fn deposit_callers_blacklist(&self) -> UnorderedSetMapper<ManagedAddress>;
}
