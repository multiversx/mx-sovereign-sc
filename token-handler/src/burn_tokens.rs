use crate::{common, err_msg};
use transaction::Operation;

#[multiversx_sc::module]
pub trait BurnTokens: utils::UtilsModule + common::storage::CommonStorage {
    #[endpoint(burnTokens)]
    fn burn_tokens(&self, operation: Operation<Self::Api>) {
        let sov_prefix = self.sov_prefix().get();

        for token in operation.tokens.iter() {
            if self.has_sov_prefix(&token.token_identifier, sov_prefix.clone()) {
                continue;
            }

            self.send().esdt_local_burn(
                &token.token_identifier,
                token.token_nonce,
                &token.token_data.amount,
            );
        }
    }
}
