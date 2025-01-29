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
    EnshrineESDTSafe,
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
            "The ESDT-Safe SC is not deployed, you skipped the third phase"
        );
    }

    fn require_phase_two_completed(&self, caller: &ManagedAddress) {
        require!(
            self.is_contract_deployed(caller, ScArray::HeaderVerifier),
            "The Header-Verifier SC is not deployed, you skipped the second phase"
        );
    }

    fn require_phase_one_completed(&self, caller: &ManagedAddress) {
        require!(
            !self.sovereigns_mapper(caller).is_empty(),
            "The current caller has not deployed any Sovereign Chain"
        );

        require!(
            self.is_contract_deployed(caller, ScArray::ChainConfig),
            "The Chain-Config SC is not deployed"
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

    fn generate_chain_id(&self, opt_preferred_chain_id: Option<ManagedBuffer>) -> ManagedBuffer {
        let mut chain_id_history_mapper = self.chain_ids();

        match opt_preferred_chain_id {
            Some(preferred_chain_id) => {
                require!(
                    !chain_id_history_mapper.contains(&preferred_chain_id),
                    "This chain ID is already used"
                );

                chain_id_history_mapper.insert(preferred_chain_id.clone());

                preferred_chain_id
            }
            None => loop {
                let new_chain_id = self.generated_random_four_char_string();
                if !chain_id_history_mapper.contains(&new_chain_id) {
                    chain_id_history_mapper.insert(new_chain_id.clone());
                    break new_chain_id;
                }
            },
        }
    }

    fn generated_random_four_char_string(&self) -> ManagedBuffer {
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
        let blockchain_api = self.blockchain();
        let caller = blockchain_api.get_caller();
        let shard_id = blockchain_api.get_shard_of_address(&caller);

        self.chain_factories(shard_id).get()
    }
}
