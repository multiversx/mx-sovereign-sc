use crate::safe_price_query;

multiversx_sc::imports!();

pub type AggregatorOutputType<M> =
    OptionalValue<MultiValue6<u32, ManagedBuffer<M>, ManagedBuffer<M>, u64, BigUint<M>, u8>>;

pub const DASH_TICKER_LEN: usize = 7;

pub struct AggregatorResult<M: ManagedTypeApi> {
    pub round_id: u32,
    pub from: ManagedBuffer<M>,
    pub to: ManagedBuffer<M>,
    pub timestamp: u64,
    pub price: BigUint<M>,
    pub decimals: u8,
}

impl<M: ManagedTypeApi> From<AggregatorOutputType<M>> for AggregatorResult<M> {
    fn from(value: AggregatorOutputType<M>) -> Self {
        let opt_value = value.into_option();
        if opt_value.is_none() {
            M::error_api_impl().signal_error(b"Invalid aggregator value");
        }

        let result = unsafe { opt_value.unwrap_unchecked() };
        let (round_id, from, to, timestamp, price, decimals) = result.into_tuple();

        Self {
            round_id,
            from,
            to,
            timestamp,
            price,
            decimals,
        }
    }
}

mod price_aggregator_proxy {
    use super::AggregatorOutputType;

    multiversx_sc::imports!();

    #[multiversx_sc::proxy]
    pub trait PriceAggregatorProxy {
        #[view(latestPriceFeedOptional)]
        fn latest_price_feed_optional(
            &self,
            from: ManagedBuffer,
            to: ManagedBuffer,
        ) -> AggregatorOutputType<Self::Api>;
    }
}

#[multiversx_sc::module]
pub trait PriceAggregatorModule: safe_price_query::SafePriceQueryModule {
    fn get_safe_price(
        &self,
        input_token_id: &TokenIdentifier,
        output_token_id: &TokenIdentifier,
    ) -> BigUint {
        let price_aggregator_address = self.price_aggregator_address().get();
        let input_ticker = self.get_token_ticker(input_token_id);
        let output_ticker = self.get_token_ticker(output_token_id);

        let agg_output: AggregatorOutputType<Self::Api> = self
            .price_aggregator_proxy(price_aggregator_address)
            .latest_price_feed_optional(input_ticker, output_ticker)
            .execute_on_dest_context();
        let result = AggregatorResult::from(agg_output);

        result.price
    }

    fn get_token_ticker(&self, token_id: &TokenIdentifier) -> ManagedBuffer {
        require!(
            token_id.is_valid_esdt_identifier(),
            "Invalid ESDT identifier"
        );

        let buffer = token_id.as_managed_buffer();
        let ticker = buffer.copy_slice(0, buffer.len() - DASH_TICKER_LEN);

        unsafe { ticker.unwrap_unchecked() }
    }

    #[storage_mapper("priceAggregatorAddress")]
    fn price_aggregator_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[proxy]
    fn price_aggregator_proxy(
        &self,
        to: ManagedAddress,
    ) -> price_aggregator_proxy::Proxy<Self::Api>;
}
