#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub mod storage;

#[multiversx_sc::module]
pub trait FeeCommonModule: crate::storage::FeeCommonStorageModule {}
