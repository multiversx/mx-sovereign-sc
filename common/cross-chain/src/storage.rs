use operation::{aliases::TxNonce, CrossChainConfig};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait CrossChainStorage {
    #[storage_mapper("lastTxNonce")]
    fn last_tx_nonce(&self) -> SingleValueMapper<TxNonce>;

    #[storage_mapper("crossChainConfig")]
    fn cross_chain_config(&self) -> SingleValueMapper<CrossChainConfig<Self::Api>>;

    #[storage_mapper("feeMarketAddress")]
    fn fee_market_address(&self) -> SingleValueMapper<ManagedAddress>;
}
