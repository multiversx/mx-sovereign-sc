use operation::{aliases::TxNonce, EsdtInfo, EsdtSafeConfig};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait CrossChainStorage {
    #[storage_mapper("lastTxNonce")]
    fn last_tx_nonce(&self) -> SingleValueMapper<TxNonce>;

    #[storage_mapper("crossChainConfig")]
    fn esdt_safe_config(&self) -> SingleValueMapper<EsdtSafeConfig<Self::Api>>;

    #[storage_mapper("feeMarketAddress")]
    fn fee_market_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("headerVerifierAddress")]
    fn header_verifier_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("sovToMxTokenId")]
    fn sovereign_to_multiversx_token_id_mapper(
        &self,
        sov_token_id: &TokenIdentifier,
    ) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("mxToSovTokenId")]
    fn multiversx_to_sovereign_token_id_mapper(
        &self,
        mx_token_id: &TokenIdentifier,
    ) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("sovEsdtTokenInfoMapper")]
    fn sovereign_to_multiversx_esdt_info_mapper(
        &self,
        token_identifier: &TokenIdentifier,
        nonce: u64,
    ) -> SingleValueMapper<EsdtInfo<Self::Api>>;

    #[storage_mapper("mxEsdtTokenInfoMapper")]
    fn multiversx_to_sovereign_esdt_info_mapper(
        &self,
        token_identifier: &TokenIdentifier,
        nonce: u64,
    ) -> SingleValueMapper<EsdtInfo<Self::Api>>;

    #[storage_mapper("isSovereignChain")]
    fn is_sovereign_chain(&self) -> SingleValueMapper<bool>;
}
