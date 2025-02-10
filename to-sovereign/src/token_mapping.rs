use cross_chain::{DEFAULT_ISSUE_COST, REGISTER_GAS};
use multiversx_sc::{require, types::EsdtTokenType};
use operation::{EsdtInfo, IssueEsdtArgs};
multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait TokenMappingModule:
    utils::UtilsModule + cross_chain::CrossChainCommon + cross_chain::storage::CrossChainStorage
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
        let issue_cost = self.call_value().egld().clone_value();
        require!(
            issue_cost == DEFAULT_ISSUE_COST,
            "EGLD value should be 0.05"
        );

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
        // NOTE: The address will be different for each specific Sovereign
        let mvx_token_id = self
            .tx()
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
            .returns(ReturnsResultUnmanaged)
            .sync_call();

        self.set_corresponding_token_ids(&args.sov_token_id, &mvx_token_id);
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
}
