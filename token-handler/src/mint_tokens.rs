use multiversx_sc::types::MultiValueEncoded;

use crate::operation::OperationEsdtPayment;

#[multiversx_sc::module]
pub trait MintTokens {
    #[endpoint(mintTokens)]
    fn mint_tokens(&self, operation_tokens: MultiValueEncoded<OperationEsdtPayment<Self::Api>>) {}
}
