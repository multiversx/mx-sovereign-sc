use multiversx_sc::{imports::SingleValueMapper, types::ManagedBuffer};

#[storage_mapper]
fn sov_prefix(&self) -> SingleValueMapper<ManagedBuffer>;
