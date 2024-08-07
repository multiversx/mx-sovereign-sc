use multiversx_sc::{imports::UnorderedSetMapper, require, types::ManagedAddress};

#[multiversx_sc::module]
pub trait CommonStorage {
    #[storage_mapper("enshrineEsdtWhitelist")]
    fn enshrine_esdt_whitelist(&self) -> UnorderedSetMapper<ManagedAddress<Self::Api>>;

    fn require_caller_to_be_whitelisted(&self) {
        let caller = self.blockchain().get_caller();
        require!(
            self.enshrine_esdt_whitelist().contains(&caller),
            "Caller is not whitelisted"
        );
    }
}
