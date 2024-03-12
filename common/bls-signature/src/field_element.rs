use multiversx_sc::{api::ManagedTypeApi, types::BigUint};


pub struct FieldElement<M: ManagedTypeApi> {
    // i64 or BigInt?
    pub value: BigUint<M>,
    pub prime: BigUint<M>
}

// impl FieldElement {
    // pub fn new(value: i64, prime: i64) -> Self {
    //     if prime < 0 {
    //         panic!("Value has to be positive");
    //     }
    //     
    //     let result = value.rem_euclid(prime);
    //
    //     FieldElement {value: result, prime}
    // }
// }

#[multiversx_sc::module]
pub trait FieldElementModule { }
