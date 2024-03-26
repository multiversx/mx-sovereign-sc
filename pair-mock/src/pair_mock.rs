#![no_std]

multiversx_sc::imports!();

pub const DEFAULT_TOKEN_PRICE: u64 = 1_000_000; // $1
pub const DEFAULT_TOKEN_DECIMALS: u32 = 18;

#[multiversx_sc::derive::contract]
pub trait PairMock {
    #[init]
    fn init(&self, usdc_token_id: TokenIdentifier) {
        self.usdc_token_id().set(usdc_token_id);
    }

    #[view(getSafePriceByTimestampOffset)]
    fn get_safe_price_by_timestamp_offset(
        &self,
        _pair_address: ManagedAddress,
        _timestamp_offset: u64,
        input_payment: EsdtTokenPayment,
    ) -> EsdtTokenPayment {
        EsdtTokenPayment::new(
            self.usdc_token_id().get(),
            0,
            input_payment.amount * DEFAULT_TOKEN_PRICE
                / BigUint::from(10u32).pow(DEFAULT_TOKEN_DECIMALS),
        )
    }

    #[storage_mapper("usdcTokenId")]
    fn usdc_token_id(&self) -> SingleValueMapper<TokenIdentifier>;
}
