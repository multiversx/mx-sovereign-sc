use crate::{common, err_msg};
use multiversx_sc::{
    hex_literal::hex,
    require,
    types::{system_proxy, ToSelf},
};
use transaction::Operation;

pub const ESDT_SYSTEM_SC_ADDRESS: [u8; 32] =
    hex!("000000000000000000010000000000000000000000000000000000000002ffff");

#[multiversx_sc::module]
pub trait BurnTokensModule: utils::UtilsModule + common::storage::CommonStorage {
    #[endpoint(burnTokens)]
    fn burn_tokens(&self, operation: Operation<Self::Api>) {
        self.require_caller_to_be_whitelisted();

        let sov_prefix = self.sov_prefix().get();

        for token in operation.tokens.iter() {
            if self.has_sov_prefix(&token.token_identifier, &sov_prefix) {
                continue;
            }

            self.tx()
                .to(ToSelf)
                .typed(system_proxy::UserBuiltinProxy)
                .esdt_local_burn(
                    &token.token_identifier,
                    token.token_nonce,
                    &token.token_data.amount,
                )
                .sync_call();
        }
    }
}
