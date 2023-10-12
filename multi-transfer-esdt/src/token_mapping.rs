multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait TokenMappingModule {
    #[only_owner]
    #[endpoint(setSovToMxTokenId)]
    fn set_sov_to_mx_token_id(&self, sov_token_id: TokenIdentifier, mx_token_id: TokenIdentifier) {
        require!(
            sov_token_id.is_valid_esdt_identifier() && mx_token_id.is_valid_esdt_identifier(),
            "Invalid token IDs"
        );

        self.sovereign_to_multiversx_token_id(&sov_token_id)
            .set(mx_token_id);
    }

    #[storage_mapper("sovToMxTokenId")]
    fn sovereign_to_multiversx_token_id(
        &self,
        sov_token_id: &TokenIdentifier,
    ) -> SingleValueMapper<TokenIdentifier>;
}
