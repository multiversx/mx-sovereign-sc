use error_messages::{
    CALLER_DID_NOT_DEPLOY_ANY_SOV_CHAIN, CHAIN_CONFIG_NOT_DEPLOYED, CHAIN_ID_ALREADY_IN_USE,
    CHAIN_ID_NOT_LOWERCASE_ALPHANUMERIC, DEPLOY_COST_NOT_ENOUGH, ESDT_SAFE_NOT_DEPLOYED,
    FEE_MARKET_NOT_DEPLOYED, HEADER_VERIFIER_NOT_DEPLOYED, INVALID_CHAIN_ID,
};
use multiversx_sc::require;
use structs::forge::ScArray;

const CHARSET: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";

use crate::err_msg;

const NUMBER_OF_SHARDS: u32 = 3;

#[multiversx_sc::module]
pub trait UtilsModule: super::storage::StorageModule {
    fn require_initialization_phase_complete(&self) {
        for shard_id in 1..=NUMBER_OF_SHARDS {
            require!(
                !self.chain_factories(shard_id).is_empty(),
                "There is no Chain-Factory contract assigned for shard {}",
                shard_id
            );
        }
    }

    fn require_phase_four_completed(&self, caller: &ManagedAddress) {
        require!(
            self.is_contract_deployed(caller, ScArray::HeaderVerifier),
            HEADER_VERIFIER_NOT_DEPLOYED
        );
    }

    fn require_phase_three_completed(&self, caller: &ManagedAddress) {
        require!(
            self.is_contract_deployed(caller, ScArray::FeeMarket),
            FEE_MARKET_NOT_DEPLOYED
        );
    }

    fn require_phase_two_completed(&self, caller: &ManagedAddress) {
        require!(
            self.is_contract_deployed(caller, ScArray::ESDTSafe),
            ESDT_SAFE_NOT_DEPLOYED
        );
    }

    fn require_phase_one_completed(&self, caller: &ManagedAddress) {
        require!(
            !self.sovereigns_mapper(caller).is_empty(),
            CALLER_DID_NOT_DEPLOY_ANY_SOV_CHAIN
        );

        require!(
            self.is_contract_deployed(caller, ScArray::ChainConfig),
            CHAIN_CONFIG_NOT_DEPLOYED
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
                self.validate_chain_id(&preferred_chain_id);

                require!(
                    !chain_id_history_mapper.contains(&preferred_chain_id),
                    CHAIN_ID_ALREADY_IN_USE
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
            DEPLOY_COST_NOT_ENOUGH
        );
    }

    fn get_chain_factory_address(&self) -> ManagedAddress {
        let blockchain_api = self.blockchain();
        let caller = blockchain_api.get_caller();
        let shard_id = blockchain_api.get_shard_of_address(&caller);

        self.chain_factories(shard_id).get()
    }

    #[inline]
    fn validate_chain_id(&self, chain_id: &ManagedBuffer) {
        let id_length = chain_id.len();
        require!(id_length >= 1 && id_length == 4, INVALID_CHAIN_ID);

        require!(
            self.is_chain_id_lowercase_alphanumeric(chain_id),
            CHAIN_ID_NOT_LOWERCASE_ALPHANUMERIC
        );
    }

    fn is_chain_id_lowercase_alphanumeric(&self, chain_id: &ManagedBuffer) -> bool {
        let mut chain_id_byte_array = [0u8; 4];
        let chain_id_byte_array = chain_id.load_to_byte_array(&mut chain_id_byte_array);

        chain_id_byte_array.iter().all(|b| CHARSET.contains(b))
    }
}
