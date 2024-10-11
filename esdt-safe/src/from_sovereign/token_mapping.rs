multiversx_sc::imports!();
multiversx_sc::derive_imports!();

const DEFAULT_ISSUE_COST: u64 = 50_000_000_000_000_000; // 0.05 EGLD

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
        self.require_token_has_prefix(&sov_token_id);
        let is_sovereign_chain = self.is_sovereign_chain().get();
        require!(
            !is_sovereign_chain,
            "Invalid method to call in current chain"
        );

        let issue_cost = self.call_value().egld_value().clone_value();
        require!(
            issue_cost == DEFAULT_ISSUE_COST,
            "eGLD value should be 0.05"
        );

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
            .to(ToCaller)
            .typed(ESDTSystemSCProxy)
            .issue_and_set_all_roles(
                args.issue_cost,
                args.token_display_name,
                args.token_ticker,
                args.token_type,
                args.num_decimals,
            )
            .gas(self.blockchain().get_gas_left())
            .callback(
                <Self as TokenMappingModule>::callbacks(self).issue_callback(&args.sov_token_id),
            )
            .register_promise();
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
            ManagedAsyncCallResult::Err(_) => {
                sc_panic!("There was an error at issuing nonfungible tokens");
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
    fn sovereign_esdt_token_esdt_mapper(
        &self,
        token_identifier: &TokenIdentifier,
        nonce: &u64,
    ) -> SingleValueMapper<EsdtInfo<Self::Api>>;

    #[storage_mapper("mxEsdtTokenInfoMapper")]
    fn multiversx_esdt_token_esdt_mapper(
        &self,
        token_identifier: &TokenIdentifier,
        nonce: &u64,
    ) -> SingleValueMapper<EsdtInfo<Self::Api>>;

    #[storage_mapper("isSovereignChain")]
    fn is_sovereign_chain(&self) -> SingleValueMapper<bool>;
}
