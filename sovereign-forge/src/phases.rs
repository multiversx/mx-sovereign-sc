use crate::err_msg;
use crate::storage::{self};

#[multiversx_sc::module]
pub trait PhasesModule: storage::StorageModule {
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
