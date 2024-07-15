#[multiversx_sc::module]
pub trait BurnTokens {
    #[endpoint(burnTokens)]
    fn burn_tokens(&self) {}
}
