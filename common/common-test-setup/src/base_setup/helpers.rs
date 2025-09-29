use multiversx_sc_scenario::multiversx_chain_vm::crypto_functions_bls::create_aggregated_signature;
use rand::RngCore;
use rand_core::OsRng;

use multiversx_sc_scenario::imports::ManagedTypeApi;
use multiversx_sc_scenario::{
    api::StaticApi,
    imports::{
        ManagedBuffer, MultiEgldOrEsdtPayment, ReturnsResultUnmanaged, TestSCAddress, TopEncode,
        UserBuiltinProxy,
    },
    multiversx_chain_vm::crypto_functions::sha256,
    ScenarioTxRun,
};
use structs::{
    forge::{ContractInfo, ScArray},
    operation::Operation,
    BLS_KEY_BYTE_LENGTH,
};

use crate::constants::NATIVE_TEST_TOKEN;
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
