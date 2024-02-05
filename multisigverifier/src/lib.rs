mod bridge_operations;

// use user_role::UserRole;

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait Multisigverifier:
   bridge_operations::BridgeOperationsModule
   // + user_roles::UserRole
{
    #[init]
    fn init(&self) {}
}
