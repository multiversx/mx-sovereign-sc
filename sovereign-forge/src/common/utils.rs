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
    fn require_phase_three_completed(&self, caller: &ManagedAddress) {
        require!(
            self.is_contract_deployed(caller, ScArray::ESDTSafe),
            "The Header-Verifier SC is not deployed, you skipped the second phase"
        );
    }

    fn require_phase_two_completed(&self, caller: &ManagedAddress) {
        require!(
            self.is_contract_deployed(caller, ScArray::HeaderVerifier),
            "The Header-Verifier SC is not deployed, you skipped the second phase"
        );
    }

    fn require_phase_1_completed(&self, caller: &ManagedAddress) {
        require!(
            !self.sovereigns_mapper(caller).is_empty(),
            "The current caller has not deployed any Sovereign Chain"
        );

        require!(
            self.is_contract_deployed(caller, ScArray::ChainConfig),
            "The Chain-Config SC is not deployed"
        );

        require!(
            !self.is_contract_deployed(caller, ScArray::HeaderVerifier),
            "The Header-Verifier SC is already deployed"
        );
    }

    fn is_contract_deployed(&self, sovereign_creator: &ManagedAddress, sc_id: ScArray) -> bool {
        let chain_id = self.sovereigns_mapper(sovereign_creator).get();
        self.sovereign_deployed_contracts(&chain_id)
            .iter()
            .any(|sc| sc.id == sc_id)
    }

    fn get_contract_address(&self, caller: &ManagedAddress, sc_id: ScArray) -> ManagedAddress {
        let chain_id = self.sovereigns_mapper(caller).get();

        self.sovereign_deployed_contracts(&chain_id)
            .iter()
            .find(|sc| sc.id == sc_id)
            .unwrap()
            .address
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
