#![no_std]

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait RouterMock {
    #[init]
    fn init(&self, pair_addr: ManagedAddress, usdc_token_id: TokenIdentifier) {
        self.pair_address().set(pair_addr);
        self.usdc_token_id().set(usdc_token_id);
    }

    #[view(getPair)]
    fn get_pair(
        &self,
        first_token_id: TokenIdentifier,
        second_token_id: TokenIdentifier,
    ) -> ManagedAddress {
        let usdc_token_id = self.usdc_token_id().get();
        if first_token_id == usdc_token_id || second_token_id == usdc_token_id {
            self.pair_address().get()
        } else {
            ManagedAddress::zero()
        }
    }

    #[storage_mapper("pairAddress")]
    fn pair_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("usdcTokenId")]
    fn usdc_token_id(&self) -> SingleValueMapper<TokenIdentifier>;
}
