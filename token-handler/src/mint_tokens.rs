use multiversx_sc::err_msg;
use multiversx_sc::{
    imports::SingleValueMapper,
    types::{ManagedVec, MultiValueEncoded},
};
use transaction::OperationEsdtPayment;

#[multiversx_sc::module]
pub trait MintTokens: utils::UtilsModule + bls_signature::BlsSignatureModule {
    #[endpoint(mintTokens)]
    fn mint_tokens(&self, operation_tokens: MultiValueEncoded<OperationEsdtPayment<Self::Api>>) {
        let output_payments = ManagedVec::new();

        for operation_token in operation_tokens {
            if self.has_sov_prefix(&operation_token.token_identifier, self.sov_prefix().get()) {}
        }
    }

    #[storage_mapper]
    fn sov_prefix(&self) -> SingleValueMapper<ManagedBuffer>;
}
