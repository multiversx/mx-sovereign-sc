#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub mod endpoints;
pub mod helpers;
pub mod storage;

#[multiversx_sc::module]
pub trait FeeCommonModule:
    crate::helpers::FeeCommonHelpersModule
    + crate::storage::FeeCommonStorageModule
    + crate::endpoints::FeeCommonEndpointsModule
    + utils::UtilsModule
    + custom_events::CustomEventsModule
{
}
