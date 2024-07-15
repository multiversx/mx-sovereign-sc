#[multiversx_sc::module]
pub trait MintTokens {
    #[endpoint(mintTokens)]
    fn mint_tokens(&self) {}
}
