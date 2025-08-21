#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait FeeCommonModule: storage::FeeCommonStorageModule {}
