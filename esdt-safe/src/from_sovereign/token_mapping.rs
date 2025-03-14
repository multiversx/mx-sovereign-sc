multiversx_sc::imports!();
multiversx_sc::derive_imports!();

const REGISTER_GAS: u64 = 60_000_000;

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, ManagedVecItem, Clone)]
pub struct EsdtInfo<M: ManagedTypeApi> {
    pub token_identifier: TokenIdentifier<M>,
    pub token_nonce: u64,
}

struct IssueEsdtArgs<M: ManagedTypeApi> {
    sov_token_id: TokenIdentifier<M>,
    token_type: EsdtTokenType,
    issue_cost: BigUint<M>,
    token_display_name: ManagedBuffer<M>,
    token_ticker: ManagedBuffer<M>,
    num_decimals: usize,
}

#[multiversx_sc::module]
pub trait TokenMappingModule: utils::UtilsModule {
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
        let is_sovereign_chain = self.is_sovereign_chain().get();
        require!(
            !is_sovereign_chain,
            "Invalid method to call in current chain"
        );

        self.require_token_has_prefix(&sov_token_id);
        self.check_for_native_token(&sov_token_id);

        let issue_cost = self.call_value().egld_value().clone_value();

        self.require_sov_token_id_not_registered(&sov_token_id);

        match token_type {
            EsdtTokenType::Invalid => sc_panic!("Invalid type"),
            _ => self.handle_token_issue(IssueEsdtArgs {
                sov_token_id: sov_token_id.clone(),
                issue_cost,
                token_display_name,
                token_ticker,
                token_type,
                num_decimals,
            }),
        }
    }

    fn handle_token_issue(&self, args: IssueEsdtArgs<Self::Api>) {
        self.tx()
            .to(ESDTSystemSCAddress)
            .typed(ESDTSystemSCProxy)
            .issue_and_set_all_roles(
                args.issue_cost,
                args.token_display_name,
                args.token_ticker,
                args.token_type,
                args.num_decimals,
            )
            .gas(REGISTER_GAS)
            .callback(
                <Self as TokenMappingModule>::callbacks(self).issue_callback(&args.sov_token_id),
            )
            .register_promise();
    }

    fn check_for_native_token(&self, token_identifier: &TokenIdentifier) {
        let native_token_mapper = self.native_token();

        if !native_token_mapper.is_empty() {
            require!(
                token_identifier == &TokenIdentifier::from(native_token_mapper.get()),
                "The current token is not the native one"
            )
        }
    }

    #[promises_callback]
    fn issue_callback(
        &self,
        sov_token_id: &TokenIdentifier,
        #[call_result] result: ManagedAsyncCallResult<TokenIdentifier<Self::Api>>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(mvx_token_id) => {
                self.set_corresponding_token_ids(sov_token_id, &mvx_token_id);
            }
            ManagedAsyncCallResult::Err(error) => {
                sc_panic!("There was an error at issuing token: '{}'", error.err_msg);
            }
        }
    }

    fn set_corresponding_token_ids(
        &self,
        sov_token_id: &TokenIdentifier,
        mvx_token_id: &TokenIdentifier,
    ) {
        self.sovereign_to_multiversx_token_id_mapper(sov_token_id)
            .set(mvx_token_id);

        self.multiversx_to_sovereign_token_id_mapper(mvx_token_id)
            .set(sov_token_id);
    }

    fn update_esdt_info_mappers(
        &self,
        sov_id: &TokenIdentifier,
        sov_nonce: u64,
        mvx_id: &TokenIdentifier,
        new_nft_nonce: u64,
    ) {
        self.sovereign_to_multiversx_esdt_info_mapper(sov_id, sov_nonce)
            .set(EsdtInfo {
                token_identifier: mvx_id.clone(),
                token_nonce: new_nft_nonce,
            });

        self.multiversx_to_sovereign_esdt_info_mapper(mvx_id, new_nft_nonce)
            .set(EsdtInfo {
                token_identifier: sov_id.clone(),
                token_nonce: sov_nonce,
            });
    }

    #[inline]
    fn clear_sov_to_mvx_esdt_info_mapper(&self, id: &TokenIdentifier, nonce: u64) {
        self.sovereign_to_multiversx_esdt_info_mapper(id, nonce)
            .take();
    }

    #[inline]
    fn clear_mvx_to_sov_esdt_info_mapper(&self, id: &TokenIdentifier, nonce: u64) {
        self.multiversx_to_sovereign_esdt_info_mapper(id, nonce)
            .take();
    }

    #[inline]
    fn is_fungible(self, token_type: &EsdtTokenType) -> bool {
        *token_type == EsdtTokenType::Fungible
    }

    #[inline]
    fn is_sft_or_meta(self, token_type: &EsdtTokenType) -> bool {
        *token_type == EsdtTokenType::SemiFungible || *token_type == EsdtTokenType::Meta
    }

    #[inline]
    fn is_nft(self, token_type: &EsdtTokenType) -> bool {
        *token_type == EsdtTokenType::NonFungible
    }

    #[inline]
    fn require_sov_token_id_not_registered(&self, id: &TokenIdentifier) {
        require!(
            self.sovereign_to_multiversx_token_id_mapper(id).is_empty(),
            "This token was already registered"
        );
    }

    #[storage_mapper("sovToMxTokenId")]
    fn sovereign_to_multiversx_token_id_mapper(
        &self,
        sov_token_id: &TokenIdentifier,
    ) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("mxToSovTokenId")]
    fn multiversx_to_sovereign_token_id_mapper(
        &self,
        mx_token_id: &TokenIdentifier,
    ) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("sovEsdtTokenInfoMapper")]
    fn sovereign_to_multiversx_esdt_info_mapper(
        &self,
        token_identifier: &TokenIdentifier,
        nonce: u64,
    ) -> SingleValueMapper<EsdtInfo<Self::Api>>;

    #[storage_mapper("mxEsdtTokenInfoMapper")]
    fn multiversx_to_sovereign_esdt_info_mapper(
        &self,
        token_identifier: &TokenIdentifier,
        nonce: u64,
    ) -> SingleValueMapper<EsdtInfo<Self::Api>>;

    #[storage_mapper("isSovereignChain")]
    fn is_sovereign_chain(&self) -> SingleValueMapper<bool>;

    #[storage_mapper("nativeToken")]
    fn native_token(&self) -> SingleValueMapper<ManagedBuffer>;
}
