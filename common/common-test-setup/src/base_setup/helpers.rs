use chain_config::storage::ChainConfigStorageModule;
use multiversx_sc::chain_core::EGLD_000000_TOKEN_IDENTIFIER;
use multiversx_sc::types::{
    BigUint, EgldOrEsdtTokenIdentifier, EgldOrEsdtTokenPayment, ManagedVec, MultiEgldOrEsdtPayment,
};
use multiversx_sc_scenario::api::{DebugApiBackend, VMHooksApi};
use multiversx_sc_scenario::multiversx_chain_vm::crypto_functions_bls::create_aggregated_signature;
use multiversx_sc_scenario::ScenarioTxWhitebox;
use rand::RngCore;
use rand_core::OsRng;

use multiversx_sc_scenario::imports::ManagedTypeApi;
use multiversx_sc_scenario::{
    api::StaticApi,
    imports::{ManagedBuffer, ReturnsResultUnmanaged, TestSCAddress, TopEncode, UserBuiltinProxy},
    multiversx_chain_vm::crypto_functions::sha256,
    ScenarioTxRun,
};
use structs::ValidatorData;
use structs::{
    forge::{ContractInfo, ScArray},
    operation::Operation,
    BLS_KEY_BYTE_LENGTH,
};

use crate::constants::{FIRST_TEST_TOKEN, NATIVE_TEST_TOKEN};
use crate::{
    base_setup::init::BaseSetup,
    constants::{
        CHAIN_CONFIG_ADDRESS, CHAIN_FACTORY_SC_ADDRESS, ESDT_SAFE_ADDRESS, FEE_MARKET_ADDRESS,
        HEADER_VERIFIER_ADDRESS, OWNER_ADDRESS,
    },
};

impl BaseSetup {
    pub fn get_native_token(&mut self) -> (ManagedBuffer<StaticApi>, ManagedBuffer<StaticApi>) {
        (NATIVE_TEST_TOKEN.as_str().into(), "Native".into())
    }
    pub fn register_multiple_validators(&mut self, new_validators: Vec<ManagedBuffer<StaticApi>>) {
        for new_validator in new_validators {
            self.register(&new_validator, &MultiEgldOrEsdtPayment::new(), None);
        }
    }

    pub fn combined_stake_payments(
        &self,
        amount: &BigUint<StaticApi>,
    ) -> MultiEgldOrEsdtPayment<StaticApi> {
        let mut payments = MultiEgldOrEsdtPayment::new();
        payments.push(EgldOrEsdtTokenPayment::new(
            EgldOrEsdtTokenIdentifier::from(EGLD_000000_TOKEN_IDENTIFIER.as_bytes()),
            0,
            amount.clone(),
        ));
        payments.push(EgldOrEsdtTokenPayment::new(
            EgldOrEsdtTokenIdentifier::from(FIRST_TEST_TOKEN.as_bytes()),
            0,
            amount.clone(),
        ));

        payments
    }

    pub fn single_token_payment(
        &self,
        amount: &BigUint<StaticApi>,
    ) -> MultiEgldOrEsdtPayment<StaticApi> {
        let mut payments = MultiEgldOrEsdtPayment::new();
        payments.push(EgldOrEsdtTokenPayment::new(
            EgldOrEsdtTokenIdentifier::from(FIRST_TEST_TOKEN.as_bytes()),
            0,
            amount.clone(),
        ));

        payments
    }

    pub fn change_ownership_to_header_verifier(&mut self, sc_address: TestSCAddress) {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(sc_address)
            .typed(UserBuiltinProxy)
            .change_owner_address(&HEADER_VERIFIER_ADDRESS.to_managed_address())
            .returns(ReturnsResultUnmanaged)
            .run();
    }

    pub fn get_operation_hash(
        &mut self,
        operation: &Operation<StaticApi>,
    ) -> ManagedBuffer<StaticApi> {
        let mut serialized_operation: ManagedBuffer<StaticApi> = ManagedBuffer::new();
        let _ = operation.top_encode(&mut serialized_operation);
        let sha256 = sha256(&serialized_operation.to_vec());

        ManagedBuffer::new_from_bytes(&sha256)
    }

    pub fn get_contract_info_struct_for_sc_type(
        &mut self,
        sc_array: Vec<ScArray>,
    ) -> Vec<ContractInfo<StaticApi>> {
        sc_array
            .iter()
            .map(|sc| {
                ContractInfo::new(
                    sc.clone(),
                    self.get_sc_address(sc.clone()).to_managed_address(),
                )
            })
            .collect()
    }

    pub fn get_sc_address(&mut self, sc_type: ScArray) -> TestSCAddress {
        match sc_type {
            ScArray::ChainConfig => CHAIN_CONFIG_ADDRESS,
            ScArray::ChainFactory => CHAIN_FACTORY_SC_ADDRESS,
            ScArray::ESDTSafe => ESDT_SAFE_ADDRESS,
            ScArray::HeaderVerifier => HEADER_VERIFIER_ADDRESS,
            ScArray::FeeMarket => FEE_MARKET_ADDRESS,
            _ => TestSCAddress::new("ERROR"),
        }
    }

    pub fn register_validators(
        &mut self,
        count: u64,
        payments: &MultiEgldOrEsdtPayment<StaticApi>,
    ) -> Vec<ManagedBuffer<StaticApi>> {
        let mut bls_keys = Vec::new();

        for expected_id in 1..=count {
            let bls_key = BLSKey::random();
            self.register(&bls_key, payments, None);
            assert_eq!(self.get_bls_key_id(&bls_key), BigUint::from(expected_id),);
            bls_keys.push(bls_key);
        }

        bls_keys
    }

    pub fn full_bitmap(&self, num_of_validators: u64) -> ManagedBuffer<StaticApi> {
        let mut bitmap_bytes = vec![0u8; num_of_validators.div_ceil(8) as usize];
        for index in 0..num_of_validators {
            let byte_index = (index / 8) as usize;
            let bit_index = (index % 8) as u8;
            bitmap_bytes[byte_index] |= 1u8 << bit_index;
        }

        ManagedBuffer::new_from_bytes(&bitmap_bytes)
    }

    /// Creates a bitmap for the given validator indices. Each value in the array represents a validator index.
    pub fn bitmap_for_signers(&self, validator_indices: &[u64]) -> ManagedBuffer<StaticApi> {
        if validator_indices.is_empty() {
            return ManagedBuffer::new_from_bytes(&[]);
        }

        let max_index = *validator_indices.iter().max().unwrap();
        let byte_len = (max_index / 8 + 1) as usize;
        let mut bitmap_bytes = vec![0u8; byte_len];

        for &validator_index in validator_indices {
            let byte_index = (validator_index / 8) as usize;
            let bit_index = (validator_index % 8) as u8;
            bitmap_bytes[byte_index] |= 1u8 << bit_index;
        }

        ManagedBuffer::new_from_bytes(&bitmap_bytes)
    }

    pub fn get_sig_and_pub_keys(
        &mut self,
        pk_size: usize,
        message: &ManagedBuffer<StaticApi>,
    ) -> (ManagedBuffer<StaticApi>, Vec<ManagedBuffer<StaticApi>>) {
        let (signature, pub_keys) =
            create_aggregated_signature(pk_size, &message.to_vec()).unwrap();
        let pk_buffers: Vec<ManagedBuffer<StaticApi>> = pub_keys
            .iter()
            .map(|pk| ManagedBuffer::from(pk.serialize().unwrap()))
            .collect();

        (
            ManagedBuffer::new_from_bytes(signature.serialize().unwrap().as_slice()),
            pk_buffers,
        )
    }

    /// Calculates the number of signers based on the bitmap.
    /// Each bit in the bitmap represents whether a validator signed.
    pub fn calculate_signer_count(&self, bitmap: &ManagedBuffer<StaticApi>) -> usize {
        let bitmap_bytes = bitmap.to_vec();
        let signer_count: usize = bitmap_bytes
            .iter()
            .map(|byte| byte.count_ones() as usize)
            .sum();
        signer_count.max(1)
    }

    pub fn update_validator_key_in_chain_config(
        &mut self,
        validator_data: &ValidatorData<StaticApi>,
        pub_keys: &[ManagedBuffer<StaticApi>],
    ) {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(CHAIN_CONFIG_ADDRESS)
            .whitebox(chain_config::contract_obj, |sc| {
                let new_pub_keys: ManagedVec<
                    VMHooksApi<DebugApiBackend>,
                    ManagedBuffer<VMHooksApi<DebugApiBackend>>,
                > = pub_keys
                    .iter()
                    .map(|pk| ManagedBuffer::new_from_bytes(&pk.to_vec()))
                    .collect();

                let validator_id = validator_data.id.to_u64().unwrap();
                let target_index = validator_id.saturating_sub(1) as usize;

                if target_index < new_pub_keys.len() {
                    let new_key = new_pub_keys.get(target_index).clone();
                    sc.validator_info(&BigUint::from(validator_id))
                        .update(|v| v.bls_key = new_key);
                }
            });
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct BLSKey([u8; BLS_KEY_BYTE_LENGTH]);

impl BLSKey {
    pub fn random<M: ManagedTypeApi>() -> ManagedBuffer<M> {
        let mut bytes = [0u8; BLS_KEY_BYTE_LENGTH];
        OsRng.fill_bytes(&mut bytes);
        ManagedBuffer::new_from_bytes(&bytes)
    }
}
