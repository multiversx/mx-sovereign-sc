use error_messages::{
    MINT_AND_BURN_ROLES_NOT_FOUND, TOKEN_ID_IS_NOT_TRUSTED, TOKEN_IS_FROM_SOVEREIGN,
};
use multiversx_sc::imports::*;

pub const TRUSTED_TOKEN_IDS: [&str; 1] = ["USDC-c76f1f"];

#[multiversx_sc::module]
pub trait BridgingMechanism:
    cross_chain::storage::CrossChainStorage + multiversx_sc_modules::only_admin::OnlyAdminModule
{
    #[only_owner]
    #[endpoint(setTokenBurnMechanism)]
    fn set_token_burn_mechanism(&self, token_id: TokenIdentifier) {
        let token_esdt_roles = self.blockchain().get_esdt_local_roles(&token_id);

        require!(
            token_esdt_roles.contains(EsdtLocalRoleFlags::MINT)
                && token_esdt_roles.contains(EsdtLocalRoleFlags::BURN),
            MINT_AND_BURN_ROLES_NOT_FOUND
        );

        require!(
            TRUSTED_TOKEN_IDS
                .iter()
                .any(|trusted_token_id| TokenIdentifier::from(*trusted_token_id) == token_id),
            TOKEN_ID_IS_NOT_TRUSTED
        );

        if self
            .multiversx_to_sovereign_token_id_mapper(&token_id)
            .is_empty()
        {
            self.burn_mechanism_tokens().insert(token_id.clone());
        }

        let sc_balance = self
            .blockchain()
            .get_sc_balance(&EgldOrEsdtTokenIdentifier::esdt(token_id.clone()), 0);

        if sc_balance != 0 {
            self.tx()
                .to(ToSelf)
                .typed(UserBuiltinProxy)
                .esdt_local_burn(&token_id, 0, &sc_balance)
                .sync_call();

            self.deposited_tokens_amount(&token_id).set(sc_balance);
        }
    }

    #[only_owner]
    #[endpoint(setTokenLockMechanism)]
    fn set_token_lock_mechanism(&self, token_id: TokenIdentifier) {
        require!(
            self.multiversx_to_sovereign_token_id_mapper(&token_id)
                .is_empty(),
            TOKEN_IS_FROM_SOVEREIGN
        );

        self.burn_mechanism_tokens().swap_remove(&token_id);

        let deposited_amount = self.deposited_tokens_amount(&token_id).get();

        if deposited_amount != 0 {
            self.tx()
                .to(ToSelf)
                .typed(UserBuiltinProxy)
                .esdt_local_mint(&token_id, 0, &deposited_amount)
                .sync_call();

            self.deposited_tokens_amount(&token_id).set(BigUint::zero());
        }
    }

    #[storage_mapper("burnMechanismTokens")]
    fn burn_mechanism_tokens(&self) -> UnorderedSetMapper<TokenIdentifier>;

    #[storage_mapper("depositedTokensAmount")]
    fn deposited_tokens_amount(
        &self,
        token_identifier: &TokenIdentifier,
    ) -> SingleValueMapper<BigUint>;
}
