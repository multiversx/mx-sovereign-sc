use pair_mock::Proxy as _;
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
pub trait SafePriceQueryModule {
    fn get_usdc_value(&self, token_payment: EsdtTokenPayment) -> BigUint {
        let pair_query_response = self.get_pair_to_query(token_payment.token_identifier);

        match pair_query_response {
            PairQueryResponse::WegldIntermediary {
                token_to_wegld_pair,
                wegld_to_usdc_pair,
            } => {
                let wegld_price = self.call_get_safe_price(token_to_wegld_pair, token_payment);

                self.call_get_safe_price(wegld_to_usdc_pair, token_payment)
            }

            PairQueryResponse::TokenToUsdc(pair_address) => {
                self.call_get_safe_price(pair_address, token_payment)
            }
        }
    }

    fn call_get_safe_price(
        &self,
        pair_address: ManagedAddress,
        token_payment: EsdtTokenPayment,
    ) -> BigUint {
        let safe_price_payment = self
            .pair_proxy(self.safe_price_pair_address().get())
            .get_safe_price_by_timestamp_offset(pair_address, HOUR_IN_SECONDS, token_payment);

        safe_price_payment.amount
    }

    fn get_pair_to_query(&self, token_id: TokenIdentifier) -> PairQueryResponse<Self::Api> {
        let wegld_token_id = self.wegld_token_id().get();
        let usdc_token_id = self.usdc_token_id().get();
        let router_address = self.router_address().get();

        let token_to_wegld_pair = self.call_get_pair(router_address, token_id, wegld_token_id);

        if !token_to_wegld_pair.is_zero() {
            let wegld_to_usdc_pair =
                self.call_get_pair(router_address, wegld_token_id, usdc_token_id);
            require!(
                !wegld_to_usdc_pair.is_zero(),
                "Invalid WEGLD-USDC pair address from router"
            );

            return PairQueryResponse::WegldIntermediary {
                token_to_wegld_pair,
                wegld_to_usdc_pair,
            };
        }

        let token_to_usdc_pair = self.call_get_pair(router_address, token_id, usdc_token_id);

        require!(
            !token_to_usdc_pair.is_zero(),
            "Invalid TOKEN-USDC pair address from router"
        );

        PairQueryResponse::TokenToUsdc(token_to_usdc_pair)
    }

    fn call_get_pair(
        &self,
        router_address: ManagedAddress,
        first_token_id: TokenIdentifier,
        second_token_id: TokenIdentifier,
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
