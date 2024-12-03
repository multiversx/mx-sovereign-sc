use multiversx_sc::{
    api::ManagedTypeApi,
    codec,
    derive::{type_abi, ManagedVecItem},
    proxy_imports::{NestedDecode, NestedEncode, TopDecode, TopEncode},
    require,
    types::ManagedAddress,
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
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, ManagedVecItem, PartialEq)]
pub enum ScArray {
    ChainFactory,
    Controller,
    HeaderVerifier,
    ESDTSafe,
    FeeMarket,
    TokenHandler,
    ChainConfig,
    Slashing,
}

#[multiversx_sc::module]
pub trait UtilsModule: super::storage::StorageModule {
    fn require_phase_1_completed(&self, caller: &ManagedAddress) {
        require!(
            self.sovereigns_mapper(caller).is_empty(),
            "The current caller has not deployed any Sovereign Chain"
        );
        self.check_if_contract_deployed(caller, ScArray::ChainConfig, b"ChainConfig");
    }

    fn check_if_contract_deployed(
        &self,
        sovereign_creator: &ManagedAddress,
        sc_id: ScArray,
        sc_name: &[u8],
    ) {
        let sovereigns_mapper = self.sovereigns_mapper(sovereign_creator);

        require!(
            !sovereigns_mapper.is_empty(),
            "There are no contracts deployed for this Sovereign"
        );

        let chain_id = sovereigns_mapper.get();
        let deployed_contracts_mapper = self.sovereign_deployed_contracts(&chain_id);

        let is_contract_deployed = deployed_contracts_mapper.iter().any(|sc| sc.id == sc_id);
        let sc_name_buffer = ManagedBuffer::from(sc_name);

        require!(
            is_contract_deployed,
            "The {} SC is not deployed inside this Sovereign Chain",
            sc_name_buffer
        );
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

    fn get_chain_factory_address(&self) -> ManagedAddress {
        let caller = self.blockchain().get_caller();
        let shard_id = self.blockchain().get_shard_of_address(&caller);

        self.chain_factories(shard_id).get()
    }
}
