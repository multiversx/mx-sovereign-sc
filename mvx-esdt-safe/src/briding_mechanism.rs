use error_messages::{
    MINT_AND_BURN_ROLES_NOT_FOUND, TOKEN_ID_IS_NOT_TRUSTED, TOKEN_IS_FROM_SOVEREIGN,
};
use multiversx_sc::imports::*;

const TRUSTED_TOKEN_IDS: [&str; 1] = ["WELGD"];

#[multiversx_sc::module]
pub trait BridgingMechanism:
    cross_chain::storage::CrossChainStorage + multiversx_sc_modules::only_admin::OnlyAdminModule
{
    #[only_admin]
    #[endpoint(setTokenBurnMechanism)]
    fn set_token_burn_mechanism(&self, token_id: TokenIdentifier) {
        require!(
            self.blockchain()
                .get_esdt_local_roles(&token_id)
                .into_iter()
                .any(|role| role == EsdtLocalRoleFlags::MINT && role == EsdtLocalRoleFlags::BURN),
            MINT_AND_BURN_ROLES_NOT_FOUND
        );

        require!(
            TRUSTED_TOKEN_IDS
                .iter()
                .any(
                    |trusted_token_id| ManagedBuffer::from(*trusted_token_id)
                        == token_id.ticker()
                ),
            TOKEN_ID_IS_NOT_TRUSTED
        );

        if !self
            .multiversx_to_sovereign_token_id_mapper(&token_id)
            .is_empty()
        {
            return;
        }

        self.burn_mechanism_tokens().insert(token_id);
    }

    #[only_admin]
    #[endpoint(setTokenLockMechanism)]
    fn set_token_lock_mechanism(&self, token_id: TokenIdentifier) {
        require!(
            self.multiversx_to_sovereign_token_id_mapper(&token_id)
                .is_empty(),
            TOKEN_IS_FROM_SOVEREIGN
        );

        self.burn_mechanism_tokens().swap_remove(&token_id);
    }

    #[storage_mapper("burnMechanismTokens")]
    fn burn_mechanism_tokens(&self) -> UnorderedSetMapper<TokenIdentifier>;
}
