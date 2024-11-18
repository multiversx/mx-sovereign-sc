#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;

mod phases;
mod storage;

#[multiversx_sc::contract]
pub trait SovereignForge {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}
}
