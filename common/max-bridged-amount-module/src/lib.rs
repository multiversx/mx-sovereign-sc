#![no_std]

use error_messages::DEPOSIT_OVER_MAX_AMOUNT;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait MaxBridgedAmountModule {}
