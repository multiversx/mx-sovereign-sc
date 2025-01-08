use multiversx_sc::imports::*;
use operation::{aliases::GasLimit, BridgeConfig};

#[multiversx_sc::module]
pub trait CommonStorage {
    #[storage_mapper("isSovereignChain")]
    fn is_sovereign_chain(&self) -> SingleValueMapper<bool>;

    #[storage_mapper("wegldIdentifier")]
    fn wegld_identifier(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("sovereignTokensPrefix")]
    fn sovereign_tokens_prefix(&self) -> SingleValueMapper<ManagedBuffer>;

    #[storage_mapper("tokenHandlerAddress")]
    fn token_handler_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("feeMarketAddress")]
    fn fee_market_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("maxUserTxGasLimit")]
    fn max_user_tx_gas_limit(&self) -> SingleValueMapper<GasLimit>;

    #[storage_mapper("bannedEndpointNames")]
    fn banned_endpoint_names(&self) -> UnorderedSetMapper<ManagedBuffer>;

    #[storage_mapper("config")]
    fn config(&self) -> SingleValueMapper<BridgeConfig<Self::Api>>;

    #[storage_mapper("headerVerifierAddress")]
    fn header_verifier_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("paidIssuedTokens")]
    fn paid_issued_tokens(&self) -> UnorderedSetMapper<TokenIdentifier<Self::Api>>;

    #[storage_mapper_from_address("feeEnabledFlag")]
    fn external_fee_enabled(
        &self,
        sc_address: ManagedAddress,
    ) -> SingleValueMapper<bool, ManagedAddress>;
}
