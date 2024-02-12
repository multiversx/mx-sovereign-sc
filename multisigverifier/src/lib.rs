mod bridge_operations;
mod utils;
// use user_role::UserRole;

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait Multisigverifier:
   bls_signature::BlsSignatureModule
   // + esdt_safe::from_sovereign::transfer_tokens::ProxyTrait
   + bridge_operations::BridgeOperationsModule
   + utils::UtilsModule
{
    #[init]
    fn init(&self, esdt_address: ManagedAddress) {
      self.esdt_proxy_address().set(esdt_address);
   }
}
