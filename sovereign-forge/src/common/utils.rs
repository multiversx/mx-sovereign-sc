use multiversx_sc::{
    api::ManagedTypeApi,
    codec,
    derive::{type_abi, ManagedVecItem},
    proxy_imports::{NestedDecode, NestedEncode, TopDecode, TopEncode},
    require,
    types::{ManagedAddress, ManagedBuffer, ManagedVec},
};

const CHARSET: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";

use crate::err_msg;

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem)]
pub struct ContractInfo<M: ManagedTypeApi> {
    pub id: ScArray,
    pub address: ManagedAddress<M>,
}

impl<M: ManagedTypeApi> ContractInfo<M> {
    pub fn new(id: ScArray, address: ManagedAddress<M>) -> Self {
        ContractInfo { id, address }
    }
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem)]
pub struct ChainContractsMap<M: ManagedTypeApi> {
    pub chain_id: ManagedBuffer<M>,
    pub contracts_info: ManagedVec<M, ContractInfo<M>>,
}

impl<M: ManagedTypeApi> ChainContractsMap<M> {
    pub fn new(chain_id: ManagedBuffer<M>, contracts_info: ManagedVec<M, ContractInfo<M>>) -> Self {
        ChainContractsMap {
            chain_id,
            contracts_info,
        }
    }
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
    fn check_phase_one_completed(&self, sovereign_creator: &ManagedAddress) {
        let sovereigns_mapper = self.sovereigns_mapper(sovereign_creator);

        require!(
            !sovereigns_mapper.is_empty(),
            "There are no contracts deployed for this Sovereign"
        );

        let last_deployed_contract = sovereigns_mapper
            .get()
            .contracts_info
            .iter()
            .last()
            .unwrap()
            .id;

        require!(last_deployed_contract == ScArray::ChainConfig, "The last deployed contract is not Chain-Config, please be attentive to the order of deployment!");
    }

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
        let mut rand = RandomnessSource::new();
        (0..4).for_each(|i| {
            let rand_index = rand.next_u8_in_range(0, CHARSET.len() as u8) as usize;
            byte_array[i] = CHARSET[rand_index];
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
