use crate::factory::ScArray;

use super::storage;
use multiversx_sc::imports::*;

#[multiversx_sc::module]
pub trait UtilsModule: storage::CommonStorage {
    fn get_contract_address(
        &self,
        chain_id: &ManagedBuffer,
        contract_name: ScArray,
    ) -> Option<ManagedAddress> {
        let deployed_contracts_mapper = self.all_deployed_contracts(chain_id.clone());

        require!(
            !deployed_contracts_mapper.is_empty(),
            "There are no contracts deployed for this sovereign chain"
        );

        let contract = deployed_contracts_mapper
            .iter()
            .find(|sc| sc.id == contract_name);

        if let Some(contract_address) = contract {
            return Some(contract_address.address);
        } else {
            return None;
        }
    }

    fn require_bls_keys_in_range(&self, chain_id: &ManagedBuffer, bls_pub_keys_count: BigUint) {
        let chain_config_address = self
            .get_contract_address(chain_id, ScArray::ChainConfig)
            .unwrap();

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

    fn generate_chain_id(&self) -> ManagedBuffer {
        loop {
            let new_chain_id = self.generated_random_4_char_string();
            if !self.chain_ids().contains(&new_chain_id) {
                self.chain_ids().insert(new_chain_id.clone());

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
