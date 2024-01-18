#![no_std]

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait UtilsModule {
    fn require_sc_address(&self, address: &ManagedAddress) {
        require!(
            !address.is_zero() && self.blockchain().is_smart_contract(address),
            "Invalid SC address"
        );
    }

    fn remove_items<
        T: TopEncode + TopDecode + NestedEncode + NestedDecode + 'static,
        I: IntoIterator<Item = T>,
    >(
        &self,
        mapper: &mut UnorderedSetMapper<T>,
        items: I,
    ) {
        for item in items {
            let was_removed = mapper.swap_remove(&item);
            require!(was_removed, "Item not found in list");
        }
    }
}
