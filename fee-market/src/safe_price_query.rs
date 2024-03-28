use crate::price_aggregator;

multiversx_sc::imports!();

pub const HOUR_IN_SECONDS: u64 = 60 * 60;

pub enum PairQueryResponse<M: ManagedTypeApi> {
    WegldIntermediary {
        token_to_wegld_pair: ManagedAddress<M>,
        wegld_to_usdc_pair: ManagedAddress<M>,
    },
    TokenToUsdc(ManagedAddress<M>),
}

#[multiversx_sc::module]
pub trait SafePriceQueryModule: price_aggregator::PriceAggregatorModule {
    fn get_usdc_value(&self, token_id: &TokenIdentifier, amount: &BigUint) -> BigUint {
        let pair_query_response = self.get_pair_to_query(token_id);

        match pair_query_response {
            PairQueryResponse::WegldIntermediary {
                token_to_wegld_pair,
                wegld_to_usdc_pair,
            } => {
                let wegld_price = self.call_get_safe_price(token_to_wegld_pair, token_id, amount);

                self.call_get_safe_price(wegld_to_usdc_pair, token_id, &wegld_price)
            }

            PairQueryResponse::TokenToUsdc(pair_address) => {
                self.call_get_safe_price(pair_address, token_id, amount)
            }
        }
    }

    fn call_get_safe_price(
        &self,
        pair_address: ManagedAddress,
        token_id: &TokenIdentifier,
        amount: &BigUint,
    ) -> BigUint {
        let safe_price_payment: EsdtTokenPayment = self
            .pair_proxy(self.safe_price_pair_address().get())
            .get_safe_price_by_timestamp_offset(
                pair_address,
                HOUR_IN_SECONDS,
                EsdtTokenPayment::new(token_id.clone(), 0, amount.clone()),
            )
            .execute_on_dest_context();

        safe_price_payment.amount
    }

    fn get_pair_to_query(&self, token_id: &TokenIdentifier) -> PairQueryResponse<Self::Api> {
        let wegld_token_id = self.wegld_token_id().get();
        let usdc_token_id = self.usdc_token_id().get();

        let token_to_wegld_pair = self.call_get_pair(&token_id, &wegld_token_id);

        if !token_to_wegld_pair.is_zero() {
            let wegld_to_usdc_pair =
                self.call_get_pair(&wegld_token_id, &usdc_token_id);
            require!(
                !wegld_to_usdc_pair.is_zero(),
                "Invalid WEGLD-USDC pair address from router"
            );

            return PairQueryResponse::WegldIntermediary {
                token_to_wegld_pair,
                wegld_to_usdc_pair,
            };
        }

        let token_to_usdc_pair = self.call_get_pair(&token_id, &usdc_token_id);

        require!(
            !token_to_usdc_pair.is_zero(),
            "Invalid TOKEN-USDC pair address from router"
        );

        PairQueryResponse::TokenToUsdc(token_to_usdc_pair)
    }

    fn call_get_pair(
        &self,
        first_token_id: &TokenIdentifier,
        second_token_id: &TokenIdentifier,
    ) -> ManagedAddress {
        self.router_proxy(self.router_address().get())
            .get_pair(first_token_id, second_token_id)
            .execute_on_dest_context()
    }

    #[proxy]
    fn router_proxy(&self, sc_address: ManagedAddress) -> router_mock::Proxy<Self::Api>;

    #[proxy]
    fn pair_proxy(&self, sc_address: ManagedAddress) -> pair_mock::Proxy<Self::Api>;

    #[storage_mapper("safePricePairAddress")]
    fn safe_price_pair_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("safePricePairAddress")]
    fn router_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("usdcTokenId")]
    fn usdc_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("wegldTokenId")]
    fn wegld_token_id(&self) -> SingleValueMapper<TokenIdentifier>;
}
