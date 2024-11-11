use crate::factory::ScArray;

use super::storage;
use multiversx_sc::imports::*;

#[multiversx_sc::module]
pub trait UtilsModule: storage::CommonStorage {
    fn is_contract_registered(
        &self,
        caller: &ManagedAddress,
        chain_id: &ManagedBuffer,
        contract_id: &ScArray,
    ) -> bool {
        let all_registered_contracts_mapper = self.all_deployed_contracts(caller);

        if all_registered_contracts_mapper.is_empty() {
            return false;
        }

        let all_contracts_info = all_registered_contracts_mapper.get();

        require!(
            *chain_id == all_contracts_info.chain_id,
            "There are no registered contracts by the caller it the {} chain",
            chain_id
        );

        all_contracts_info
            .contracts_info
            .iter()
            .any(|sc_info| &sc_info.id == contract_id)
    }

    fn require_bls_keys_in_range(
        &self,
        caller: &ManagedAddress,
        chain_id: &ManagedBuffer,
        bls_pub_keys_count: BigUint,
    ) {
        let chain_config_address =
            self.get_contract_address(caller, chain_id, &ScArray::ChainConfig);

        require!(
            !chain_config_address.is_zero(),
            "The Chain Config contract was not deployed"
        );

        let min_validators = self
            .external_min_validators(chain_config_address.clone())
            .get();
        let max_validators = self.external_max_validators(chain_config_address).get();

        require!(
            min_validators < bls_pub_keys_count && bls_pub_keys_count < max_validators,
            "The number of validator BLS Keys is not correct"
        );
    }

    fn get_contract_address(
        &self,
        caller: &ManagedAddress,
        chain_id: &ManagedBuffer,
        contract_id: &ScArray,
    ) -> ManagedAddress {
        let all_deployed_contracts_mapper = self.all_deployed_contracts(caller);

        require!(
            !all_deployed_contracts_mapper.is_empty(),
            "There are no registered contracts in the {} chain",
            chain_id
        );

        all_deployed_contracts_mapper
            .get()
            .contracts_info
            .iter()
            .find(|sc_info| &sc_info.id == contract_id)
            .unwrap_or_else(|| {
                let contract_id_u32 = contract_id.clone() as u32;
                sc_panic!("The contract with {} id was not deployed", contract_id_u32);
            })
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
        let charset: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
        let mut rand = RandomnessSource::new();

        (0..4).for_each(|i| {
            let rand_index = rand.next_u8_in_range(0, charset.len() as u8) as usize;
            byte_array[i] = charset[rand_index];
        });

        ManagedBuffer::new_from_bytes(&byte_array)
    }
}
