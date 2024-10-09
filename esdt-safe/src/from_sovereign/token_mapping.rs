multiversx_sc::imports!();
multiversx_sc::derive_imports!();

const DEFAULT_ISSUE_COST: u64 = 50_000_000_000_000_000; // 0.05 EGLD

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, ManagedVecItem, Clone)]
pub struct EsdtTokenInfo<M: ManagedTypeApi> {
    pub token_identifier: TokenIdentifier<M>,
    pub token_nonce: u64,
}

struct FungibleTokenArgs<M: ManagedTypeApi> {
    sov_token_id: TokenIdentifier<M>,
    issue_cost: BigUint<M>,
    token_display_name: ManagedBuffer<M>,
    token_ticker: ManagedBuffer<M>,
    num_decimals: usize,
}

struct NonFungibleTokenArgs<M: ManagedTypeApi> {
    sov_token_id: TokenIdentifier<M>,
    token_type: EsdtTokenType,
    issue_cost: BigUint<M>,
    token_display_name: ManagedBuffer<M>,
    token_ticker: ManagedBuffer<M>,
    num_decimals: usize,
}

#[multiversx_sc::module]
pub trait TokenMappingModule {
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
        require!(
            !self.is_sovereign_chain().get(),
            "Invalid method to call in current chain"
        );

        let issue_cost = self.call_value().egld_value().clone_value();
        require!(
            issue_cost == DEFAULT_ISSUE_COST,
            "eGLD value should be 0.05"
        );

        match token_type {
            EsdtTokenType::Invalid => sc_panic!("Invalid type"),
            EsdtTokenType::Fungible => self.handle_fungible_token_type(FungibleTokenArgs {
                sov_token_id: sov_token_id.clone(),
                issue_cost,
                token_display_name,
                token_ticker,
                num_decimals,
            }),
            _ => self.handle_nonfungible_token_type(NonFungibleTokenArgs {
                sov_token_id: sov_token_id.clone(),
                token_type,
                issue_cost,
                token_display_name,
                token_ticker,
                num_decimals,
            }),
        }

        // !!! Unreachable code !!!
        match self.sovereign_to_multiversx_token_id(&sov_token_id).get() {
            TokenMapperState::NotSet => sc_panic!("Token ID not set"),
            TokenMapperState::Pending => {}
            TokenMapperState::Token(mx_token_id) => {
                self.multiversx_to_sovereign_token_id(&mx_token_id)
                    .set(sov_token_id);
            }
        }
    }

    fn handle_fungible_token_type(&self, args: FungibleTokenArgs<Self::Api>) {
        self.multiversx_to_sovereign_token_id(&args.sov_token_id)
            .set(args.sov_token_id.clone());

        self.fungible_token(&args.sov_token_id)
            .issue_and_set_all_roles(
                args.issue_cost,
                args.token_display_name,
                args.token_ticker,
                args.num_decimals,
                None,
            );
    }

    fn handle_nonfungible_token_type(&self, args: NonFungibleTokenArgs<Self::Api>) {
        self.multiversx_to_sovereign_token_id(&args.sov_token_id)
            .set(args.sov_token_id.clone());

        self.non_fungible_token(&args.sov_token_id)
            .issue_and_set_all_roles(
                args.token_type,
                args.issue_cost,
                args.token_display_name,
                args.token_ticker,
                args.num_decimals,
                None,
            );
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
