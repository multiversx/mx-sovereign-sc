use multiversx_sc::{
    api::ManagedTypeApi,
    codec,
    derive::{type_abi, ManagedVecItem, TypeAbi},
    proxy_imports::{NestedDecode, NestedEncode, TopDecode, TopEncode},
    require,
    types::{ManagedAddress, ManagedBuffer, ManagedVec},
};

use crate::err_msg;

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem)]
pub struct ContractInfo<M: ManagedTypeApi> {
    pub id: ScArray,
    pub address: ManagedAddress<M>,
}

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem)]
pub struct ChainContractsMap<M: ManagedTypeApi> {
    pub chain_id: ManagedBuffer<M>,
    pub contracts_info: ManagedVec<M, ContractInfo<M>>,
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, ManagedVecItem, PartialEq)]
pub enum ScArray {
    ChainFactory,
    Controller,
    SovereignHeaderVerifier,
    SovereignCrossChainOperation,
    FeeMarket,
    TokenHandler,
    ChainConfig,
    Slashing,
}

#[multiversx_sc::module]
pub trait UtilsModule: super::storage::StorageModule {
    fn generate_chain_id(&self) -> ManagedBuffer {
        loop {
            let new_chain_id = self.generated_random_4_char_string();
            let mut chain_id_history_mapper = self.chain_ids();
            if !chain_id_history_mapper.contains(&new_chain_id) {
                chain_id_history_mapper.insert(new_chain_id.clone());
                return new_chain_id;
            }
        }
    }

    fn generated_random_4_char_string(&self) -> ManagedBuffer {
        let mut byte_array: [u8; 4] = [0; 4];
        let charset: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
        let mut rand = RandomnessSource::new();
        (0..4).for_each(|i| {
            let rand_index = rand.next_u8_in_range(0, charset.len() as u8) as usize;
            byte_array[i] = charset[rand_index];
        });
        ManagedBuffer::new_from_bytes(&byte_array)
    }

    fn require_correct_deploy_cost(&self, call_value: &BigUint) {
        require!(
            call_value == &self.deploy_cost().get(),
            "The given deploy cost is not equal to the standard amount"
        );
    }
}
