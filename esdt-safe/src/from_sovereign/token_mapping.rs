const DEFAULT_ISSUE_COST: u64 = 5000000000000000000;
multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait TokenMappingModule:
    multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[payable("EGLD")]
    #[endpoint(registerToken)]
    fn register_token(
        &self,
        sov_token_id: TokenIdentifier,
        token_type: EsdtTokenType,
        token_display_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
        num_decimals: usize,
    ) {
        let mut serialized_data = ManagedBuffer::new();
        let _ = sov_token_id.dep_encode(&mut serialized_data);
        let _ = token_type.dep_encode(&mut serialized_data);

        let issue_cost = self.call_value().egld_value().clone_value();

        require!(
            issue_cost == BigUint::from(DEFAULT_ISSUE_COST),
            "eGLD value should be 0.5"
        );

        match token_type {
            EsdtTokenType::Invalid => sc_panic!("Invalid type"),
            EsdtTokenType::Fungible => self.fungible_token(&sov_token_id).issue_and_set_all_roles(
                issue_cost,
                token_display_name,
                token_ticker,
                num_decimals,
                None,
            ),
            _ => self
                .non_fungible_token(&sov_token_id)
                .issue_and_set_all_roles(
                    token_type,
                    issue_cost,
                    token_display_name,
                    token_ticker,
                    num_decimals,
                    None,
                ),
        }
    }

    #[only_owner]
    #[endpoint(clearRegisteredToken)]
    fn clear_registered_token(&self, sov_token_id: TokenIdentifier) {
        self.sovereign_to_multiversx_token_id(&sov_token_id).clear();
    }

    // WARNING: All mappers must have the exact same storage key!

    #[storage_mapper("sovToMxTokenId")]
    fn sovereign_to_multiversx_token_id(
        &self,
        sov_token_id: &TokenIdentifier,
    ) -> SingleValueMapper<TokenMapperState<Self::Api>>;

    #[storage_mapper("sovToMxTokenId")]
    fn fungible_token(&self, sov_token_id: &TokenIdentifier) -> FungibleTokenMapper;

    #[storage_mapper("sovToMxTokenId")]
    fn non_fungible_token(&self, sov_token_id: &TokenIdentifier) -> NonFungibleTokenMapper;
}
