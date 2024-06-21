multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, ManagedVecItem, Clone)]
pub struct EsdtTokenInfo<M: ManagedTypeApi> {
    pub token_identifier: TokenIdentifier<M>,
    pub token_nonce: u64,
}

#[multiversx_sc::module]
pub trait TokenMappingModule:
    multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    fn handle_fungible_token_type(
        &self,
        sov_token_id: TokenIdentifier,
        issue_cost: BigUint,
        token_display_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
        num_decimals: usize,
    ) {
        self.multiversx_to_sovereign_token_id(&sov_token_id)
            .set(sov_token_id.clone());

        self.fungible_token(&sov_token_id).issue_and_set_all_roles(
            issue_cost,
            token_display_name,
            token_ticker,
            num_decimals,
            None,
        );
    }

    fn handle_nonfungible_token_type(
        &self,
        sov_token_id: TokenIdentifier,
        token_type: EsdtTokenType,
        issue_cost: BigUint,
        token_display_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
        num_decimals: usize,
    ) {
        self.multiversx_to_sovereign_token_id(&sov_token_id)
            .set(sov_token_id.clone());

        self.non_fungible_token(&sov_token_id)
            .issue_and_set_all_roles(
                token_type,
                issue_cost,
                token_display_name,
                token_ticker,
                num_decimals,
                None,
            );
    }

    #[only_owner]
    #[endpoint(clearRegisteredSovereignToken)]
    fn clear_registered_sovereign_token(&self, sov_token_id: TokenIdentifier) {
        self.sovereign_to_multiversx_token_id(&sov_token_id).clear();
    }

    #[only_owner]
    #[endpoint(clearRegisteredMultiversxToken)]
    fn clear_registered_multiversx_token(&self, mvx_token_id: TokenIdentifier) {
        self.multiversx_to_sovereign_token_id(&mvx_token_id).clear();
    }

    #[storage_mapper("sovToMxTokenId")]
    fn sovereign_to_multiversx_token_id(
        &self,
        sov_token_id: &TokenIdentifier,
    ) -> SingleValueMapper<TokenMapperState<Self::Api>>;

    #[storage_mapper("mxToSovTokenId")]
    fn multiversx_to_sovereign_token_id(
        &self,
        mx_token_id: &TokenIdentifier,
    ) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("sovToMxTokenId")]
    fn fungible_token(&self, sov_token_id: &TokenIdentifier) -> FungibleTokenMapper;

    #[storage_mapper("sovToMxTokenId")]
    fn non_fungible_token(&self, sov_token_id: &TokenIdentifier) -> NonFungibleTokenMapper;

    #[storage_mapper("sovEsdtTokenInfoMapper")]
    fn sovereign_esdt_token_info_mapper(
        &self,
        token_identifier: &TokenIdentifier,
        nonce: &u64,
    ) -> SingleValueMapper<EsdtTokenInfo<Self::Api>>;

    #[storage_mapper("mxEsdtTokenInfoMapper")]
    fn multiversx_esdt_token_info_mapper(
        &self,
        token_identifier: &TokenIdentifier,
        nonce: &u64,
    ) -> SingleValueMapper<EsdtTokenInfo<Self::Api>>;

    #[storage_mapper("isSovereignChain")]
    fn is_sovereign_chain(&self) -> SingleValueMapper<bool>;
}
