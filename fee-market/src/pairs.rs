multiversx_sc::imports!();

mod pair_proxy {
    multiversx_sc::imports!();

    #[multiversx_sc::proxy]
    pub trait PairProxy {
        #[view(getSafePriceByDefaultOffset)]
        fn get_safe_price_by_default_offset(
            &self,
            pair_address: ManagedAddress,
            input_payment: EsdtTokenPayment,
        ) -> EsdtTokenPayment;
    }
}

#[multiversx_sc::module]
pub trait PairsModule {
    #[only_owner]
    #[endpoint(addPairs)]
    fn add_pairs(
        &self,
        tokens_address_pairs: MultiValueEncoded<
            MultiValue3<TokenIdentifier, TokenIdentifier, ManagedAddress>,
        >,
    ) {
        for pairs in tokens_address_pairs {
            let (first_token_id, second_token_id, pair_address) = pairs.into_tuple();
            self.pair_address(&first_token_id, &second_token_id)
                .set(&pair_address);
        }
    }

    #[only_owner]
    #[endpoint(removePairs)]
    fn remove_pairs(
        &self,
        tokens_pairs: MultiValueEncoded<MultiValue2<TokenIdentifier, TokenIdentifier>>,
    ) {
        for pairs in tokens_pairs {
            let (first_token_id, second_token_id) = pairs.into_tuple();
            self.pair_address(&first_token_id, &second_token_id).clear();
            self.pair_address(&second_token_id, &first_token_id).clear();
        }
    }

    fn get_safe_price(
        &self,
        input_token: EsdtTokenPayment,
        output_token_id: &TokenIdentifier,
    ) -> BigUint {
        let pair_for_query = self.pair_for_query().get();
        let pair_addr = self.get_pair_address(&input_token.token_identifier, output_token_id);

        let output_token: EsdtTokenPayment = self
            .pair_proxy(pair_for_query)
            .get_safe_price_by_default_offset(pair_addr, input_token)
            .execute_on_dest_context();

        require!(
            &output_token.token_identifier == output_token_id,
            "Invalid token, wrong pair setup"
        );

        output_token.amount
    }

    fn get_pair_address(
        &self,
        first_token_id: &TokenIdentifier,
        second_token_id: &TokenIdentifier,
    ) -> ManagedAddress {
        let correct_order_mapper = self.pair_address(first_token_id, second_token_id);
        if !correct_order_mapper.is_empty() {
            return correct_order_mapper.get();
        }

        let reverse_order_mapper = self.pair_address(second_token_id, first_token_id);
        require!(!reverse_order_mapper.is_empty(), "No pair known for tokens");

        reverse_order_mapper.get()
    }

    #[storage_mapper("pairForQuery")]
    fn pair_for_query(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("pairAddress")]
    fn pair_address(
        &self,
        first_token_id: &TokenIdentifier,
        second_token_id: &TokenIdentifier,
    ) -> SingleValueMapper<ManagedAddress>;

    #[proxy]
    fn pair_proxy(&self, to: ManagedAddress) -> pair_proxy::Proxy<Self::Api>;
}
