use error_messages::{
    BURN_MECHANISM_NON_ESDT_TOKENS, LOCK_MECHANISM_NON_ESDT, MINT_AND_BURN_ROLES_NOT_FOUND,
    SETUP_PHASE_ALREADY_COMPLETED, SETUP_PHASE_NOT_COMPLETED,
    TOKEN_ALREADY_REGISTERED_WITH_BURN_MECHANISM, TOKEN_ID_IS_NOT_TRUSTED,
    TOKEN_NOT_REGISTERED_WITH_BURN_MECHANISM,
};
use multiversx_sc::imports::*;
use structs::{
    configs::{SetBurnMechanismOperation, SetLockMechanismOperation},
    generate_hash::GenerateHash,
};

#[multiversx_sc::module]
pub trait BridgingMechanism:
    cross_chain::storage::CrossChainStorage
    + setup_phase::SetupPhaseModule
    + common_utils::CommonUtilsModule
    + custom_events::CustomEventsModule
{
    #[only_owner]
    #[endpoint(setTokenBurnMechanismSetupPhase)]
    fn set_token_burn_mechanism_setup_phase(&self, token_id: EgldOrEsdtTokenIdentifier) {
        require!(
            !self.is_setup_phase_complete(),
            SETUP_PHASE_ALREADY_COMPLETED
        );
        let mut burn_mechanism_tokens_mapper = self.burn_mechanism_tokens();
        require!(
            !burn_mechanism_tokens_mapper.contains(&token_id),
            TOKEN_ALREADY_REGISTERED_WITH_BURN_MECHANISM
        );
        let esdt_identifier = token_id.clone().unwrap_esdt();
        let token_esdt_roles = self.blockchain().get_esdt_local_roles(&esdt_identifier);

        require!(
            token_esdt_roles.contains(EsdtLocalRoleFlags::MINT)
                && token_esdt_roles.contains(EsdtLocalRoleFlags::BURN),
            MINT_AND_BURN_ROLES_NOT_FOUND
        );

        require!(
            self.trusted_tokens(self.sovereign_forge_address().get())
                .iter()
                .any(|trusted_token_id| TokenIdentifier::from(trusted_token_id) == token_id),
            TOKEN_ID_IS_NOT_TRUSTED
        );

        burn_mechanism_tokens_mapper.insert(token_id.clone());
        let sc_balance = self
            .blockchain()
            .get_sc_balance(&EgldOrEsdtTokenIdentifier::esdt(esdt_identifier.clone()), 0);

        if sc_balance != 0 {
            self.tx()
                .to(ToSelf)
                .typed(UserBuiltinProxy)
                .esdt_local_burn(&esdt_identifier, 0, &sc_balance)
                .sync_call();

            self.deposited_tokens_amount(&token_id).set(sc_balance);
        }
    }

    #[endpoint(setTokenBurnMechanism)]
    fn set_token_burn_mechanism(
        &self,
        hash_of_hashes: ManagedBuffer,
        set_burn_mechanism_operation: SetBurnMechanismOperation<Self::Api>,
    ) {
        let operation_hash = set_burn_mechanism_operation.generate_hash();
        if let Some(error_message) = self.validate_operation_hash(&operation_hash) {
            self.complete_operation(&hash_of_hashes, &operation_hash, Some(error_message));
            return;
        }
        if !self.is_setup_phase_complete() {
            self.complete_operation(
                &hash_of_hashes,
                &operation_hash,
                Some(SETUP_PHASE_NOT_COMPLETED.into()),
            );
            return;
        }
        if !set_burn_mechanism_operation.token_id.is_esdt() {
            self.complete_operation(
                &hash_of_hashes,
                &operation_hash,
                Some(BURN_MECHANISM_NON_ESDT_TOKENS.into()),
            );
            return;
        }

        let mut burn_mechanism_tokens_mapper = self.burn_mechanism_tokens();
        if burn_mechanism_tokens_mapper.contains(&set_burn_mechanism_operation.token_id) {
            self.complete_operation(
                &hash_of_hashes,
                &operation_hash,
                Some(TOKEN_ALREADY_REGISTERED_WITH_BURN_MECHANISM.into()),
            );
            return;
        }

        let token_identifier = set_burn_mechanism_operation.token_id.clone().unwrap_esdt();
        let token_esdt_roles = self.blockchain().get_esdt_local_roles(&token_identifier);

        if !(token_esdt_roles.contains(EsdtLocalRoleFlags::MINT)
            && token_esdt_roles.contains(EsdtLocalRoleFlags::BURN))
        {
            self.complete_operation(
                &hash_of_hashes,
                &operation_hash,
                Some(MINT_AND_BURN_ROLES_NOT_FOUND.into()),
            );
            return;
        }
        if !self
            .trusted_tokens(self.sovereign_forge_address().get())
            .iter()
            .any(|trusted_token_id| {
                TokenIdentifier::from(trusted_token_id) == set_burn_mechanism_operation.token_id
            })
        {
            self.complete_operation(
                &hash_of_hashes,
                &operation_hash,
                Some(TOKEN_ID_IS_NOT_TRUSTED.into()),
            );
            return;
        }

        if let Some(lock_operation_error) = self.lock_operation_hash_wrapper(
            &hash_of_hashes,
            &operation_hash,
            set_burn_mechanism_operation.nonce,
        ) {
            self.complete_operation(&hash_of_hashes, &operation_hash, Some(lock_operation_error));
            return;
        }

        burn_mechanism_tokens_mapper.insert(set_burn_mechanism_operation.token_id.clone());
        let sc_balance = self.blockchain().get_sc_balance(
            &EgldOrEsdtTokenIdentifier::esdt(token_identifier.clone()),
            0,
        );

        if sc_balance != 0 {
            self.tx()
                .to(ToSelf)
                .typed(UserBuiltinProxy)
                .esdt_local_burn(&token_identifier, 0, &sc_balance)
                .sync_call();

            self.deposited_tokens_amount(&set_burn_mechanism_operation.token_id)
                .set(sc_balance);
        }

        self.complete_operation(&hash_of_hashes, &operation_hash, None);
    }

    #[only_owner]
    #[endpoint(setTokenLockMechanismSetupPhase)]
    fn set_token_lock_mechanism_setup_phase(&self, token_id: EgldOrEsdtTokenIdentifier<Self::Api>) {
        require!(
            !self.is_setup_phase_complete(),
            SETUP_PHASE_ALREADY_COMPLETED
        );
        require!(
            self.burn_mechanism_tokens().contains(&token_id),
            TOKEN_NOT_REGISTERED_WITH_BURN_MECHANISM
        );
        require!(token_id.is_esdt(), LOCK_MECHANISM_NON_ESDT);

        self.burn_mechanism_tokens().swap_remove(&token_id);

        let deposited_amount = self.deposited_tokens_amount(&token_id).get();

        if deposited_amount != 0 {
            self.tx()
                .to(ToSelf)
                .typed(UserBuiltinProxy)
                .esdt_local_mint(token_id.clone().unwrap_esdt(), 0, &deposited_amount)
                .sync_call();

            self.deposited_tokens_amount(&token_id).set(BigUint::zero());
        }
    }

    #[endpoint(setTokenLockMechanism)]
    fn set_token_lock_mechanism(
        &self,
        hash_of_hashes: ManagedBuffer,
        set_lock_mechanism_operation: SetLockMechanismOperation<Self::Api>,
    ) {
        let operation_hash = set_lock_mechanism_operation.generate_hash();
        if let Some(error_message) = self.validate_operation_hash(&operation_hash) {
            self.complete_operation(&hash_of_hashes, &operation_hash, Some(error_message));
            return;
        }
        if !self.is_setup_phase_complete() {
            self.complete_operation(
                &hash_of_hashes,
                &operation_hash,
                Some(SETUP_PHASE_NOT_COMPLETED.into()),
            );
            return;
        }
        if !set_lock_mechanism_operation.token_id.is_esdt() {
            self.complete_operation(
                &hash_of_hashes,
                &operation_hash,
                Some(LOCK_MECHANISM_NON_ESDT.into()),
            );
            return;
        }

        let mut burn_mechanism_tokens_mapper = self.burn_mechanism_tokens();
        if !burn_mechanism_tokens_mapper.contains(&set_lock_mechanism_operation.token_id) {
            self.complete_operation(
                &hash_of_hashes,
                &operation_hash,
                Some(TOKEN_NOT_REGISTERED_WITH_BURN_MECHANISM.into()),
            );
            return;
        }

        if let Some(lock_operation_error) = self.lock_operation_hash_wrapper(
            &hash_of_hashes,
            &operation_hash,
            set_lock_mechanism_operation.nonce,
        ) {
            self.complete_operation(&hash_of_hashes, &operation_hash, Some(lock_operation_error));
            return;
        }

        burn_mechanism_tokens_mapper.swap_remove(&set_lock_mechanism_operation.token_id);

        let deposited_amount = self
            .deposited_tokens_amount(&set_lock_mechanism_operation.token_id)
            .get();

        if deposited_amount != 0 {
            self.tx()
                .to(ToSelf)
                .typed(UserBuiltinProxy)
                .esdt_local_mint(
                    set_lock_mechanism_operation.token_id.clone().unwrap_esdt(),
                    0,
                    &deposited_amount,
                )
                .sync_call();

            self.deposited_tokens_amount(&set_lock_mechanism_operation.token_id)
                .set(BigUint::zero());
        }

        self.complete_operation(&hash_of_hashes, &operation_hash, None);
    }

    #[storage_mapper_from_address("trustedTokens")]
    fn trusted_tokens(
        &self,
        sc_address: ManagedAddress,
    ) -> UnorderedSetMapper<ManagedBuffer, ManagedAddress>;

    #[storage_mapper("sovereignForgeAddress")]
    fn sovereign_forge_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("burnMechanismTokens")]
    fn burn_mechanism_tokens(&self) -> UnorderedSetMapper<EgldOrEsdtTokenIdentifier<Self::Api>>;

    #[view(getDepositedTokensAmount)]
    #[storage_mapper("depositedTokensAmount")]
    fn deposited_tokens_amount(
        &self,
        token_identifier: &EgldOrEsdtTokenIdentifier<Self::Api>,
    ) -> SingleValueMapper<BigUint>;
}
