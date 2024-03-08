pub struct FieldElement {
    value: i64,
    prime: i64
}

impl FieldElement {
    pub fn new(value: i64, prime: i64) -> Self {
        if prime < 0 {
            panic!("Value has to be positive");
        }
        
        let result = value.rem_euclid(prime);

        FieldElement {value: result, prime}
    }
}

#[multiversx_sc::module]
pub trait FieldElementModule { }
