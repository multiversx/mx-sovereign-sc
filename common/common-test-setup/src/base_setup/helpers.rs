use multiversx_sc_scenario::{
    api::StaticApi,
    imports::{ManagedBuffer, ReturnsResultUnmanaged, TestSCAddress, TopEncode, UserBuiltinProxy},
    multiversx_chain_vm::crypto_functions::sha256,
    ScenarioTxRun,
};
use structs::{
    forge::{ContractInfo, ScArray},
    operation::Operation,
};

use crate::{
    base_setup::init::BaseSetup,
    constants::{
        CHAIN_CONFIG_ADDRESS, CHAIN_FACTORY_SC_ADDRESS, ENSHRINE_SC_ADDRESS, ESDT_SAFE_ADDRESS,
        FEE_MARKET_ADDRESS, HEADER_VERIFIER_ADDRESS, OWNER_ADDRESS,
    },
};

impl BaseSetup {
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
            ScArray::EnshrineESDTSafe => ENSHRINE_SC_ADDRESS,
            _ => TestSCAddress::new("ERROR"),
        }
    }
}
