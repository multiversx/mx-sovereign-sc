#![no_std]

use structs::forge::ContractInfo;
pub mod checks;
pub mod operations;
pub mod storage;
pub mod utils;

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait Headerverifier:
    storage::HeaderVerifierStorageModule
    + utils::HeaderVerifierUtilsModule
    + operations::HeaderVerifierOperationsModule
    + checks::HeaderVerifierChecksModule
    + custom_events::CustomEventsModule
    + setup_phase::SetupPhaseModule
{
    #[init]
    fn init(&self, sovereign_contracts: MultiValueEncoded<ContractInfo<Self::Api>>) {
        self.sovereign_contracts().extend(sovereign_contracts);
    }

    #[upgrade]
    fn upgrade(&self) {}

    #[only_owner]
    #[endpoint(completeSetupPhase)]
    fn complete_setup_phase(&self) {
        if self.is_setup_phase_complete() {
            return;
        }
        let chain_config_address = self.get_chain_config_address();

        self.require_chain_config_setup_complete(&chain_config_address);

        let genesis_validators: ManagedVec<ManagedBuffer> = self
            .bls_keys_map(chain_config_address)
            .iter()
            .map(|(_, bls_key)| bls_key)
            .collect();

        self.bls_pub_keys(0).extend(genesis_validators);

        self.setup_phase_complete().set(true);
    }
}
