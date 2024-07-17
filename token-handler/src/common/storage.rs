use multiversx_sc::imports::{SingleValueMapper, VecMapper};
use transaction::OperationEsdtPayment;

#[multiversx_sc::module]
pub trait CommonStorage {
    #[storage_mapper]
    fn sov_prefix(&self) -> SingleValueMapper<ManagedBuffer>;

    #[storage_mapper("mintedTokens")]
    fn minted_tokens(&self) -> VecMapper<OperationEsdtPayment<Self::Api>>;
}
