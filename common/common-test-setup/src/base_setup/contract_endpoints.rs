use crate::constants::EXECUTED_BRIDGE_OP_EVENT;
use crate::{
    base_setup::init::BaseSetup,
    constants::{CHAIN_CONFIG_ADDRESS, FEE_MARKET_ADDRESS, HEADER_VERIFIER_ADDRESS, OWNER_ADDRESS},
};

use chain_config::storage::ChainConfigStorageModule;
use header_verifier::storage::HeaderVerifierStorageModule;
use multiversx_sc_scenario::api::{DebugApiBackend, VMHooksApi};
use multiversx_sc_scenario::imports::{BigUint, ManagedVec, ReturnsResult, StorageClearable};
use multiversx_sc_scenario::multiversx_chain_vm::crypto_functions::sha256;
use multiversx_sc_scenario::ScenarioTxWhitebox;
use multiversx_sc_scenario::{
    api::StaticApi,
    imports::{
        ManagedBuffer, MultiEgldOrEsdtPayment, MultiValueEncoded, ReturnsHandledOrError,
        TestAddress,
    },
    ReturnsLogs, ScenarioTxRun,
};
use proxies::{
    chain_config_proxy::ChainConfigContractProxy, header_verifier_proxy::HeaderverifierProxy,
    mvx_fee_market_proxy::MvxFeeMarketProxy,
};
use structs::aliases::TxNonce;
use structs::fee::FeeStruct;
use structs::generate_hash::GenerateHash;
use structs::{ValidatorData, ValidatorOperation};

impl BaseSetup {
    pub fn register_operation(
        &mut self,
        caller: TestAddress,
        signature: ManagedBuffer<StaticApi>,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        bitmap: ManagedBuffer<StaticApi>,
        epoch: u64,
        operations_hashes: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>>,
    ) {
        self.world
            .tx()
            .from(caller)
            .to(HEADER_VERIFIER_ADDRESS)
            .typed(HeaderverifierProxy)
            .register_bridge_operations(signature, hash_of_hashes, bitmap, epoch, operations_hashes)
            .run();
    }

    pub fn set_fee_during_setup_phase(
        &mut self,
        fee_struct: FeeStruct<StaticApi>,
        error_message: Option<&str>,
    ) {
        let response = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(FEE_MARKET_ADDRESS)
            .typed(MvxFeeMarketProxy)
            .set_fee_during_setup_phase(fee_struct)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.assert_expected_error_message(response, error_message);
    }

    pub fn register(
        &mut self,
        bls_key: &ManagedBuffer<StaticApi>,
        payment: &MultiEgldOrEsdtPayment<StaticApi>,
        expected_error_message: Option<&str>,
    ) {
        let (response, _) = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(CHAIN_CONFIG_ADDRESS)
            .typed(ChainConfigContractProxy)
            .register(bls_key)
            .payment(payment)
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run();

        self.assert_expected_error_message(response, expected_error_message);
    }

    pub fn unregister(&mut self, bls_key: &ManagedBuffer<StaticApi>, expect_error: Option<&str>) {
        let (response, _) = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(CHAIN_CONFIG_ADDRESS)
            .typed(ChainConfigContractProxy)
            .unregister(bls_key)
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run();

        self.assert_expected_error_message(response, expect_error);
    }

    pub fn register_validator(
        &mut self,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        validator_data: ValidatorData<StaticApi>,
        operation_nonce: TxNonce,
        expected_custom_log: Option<&str>,
        expected_error_log: Option<&str>,
    ) {
        let (response, logs) = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(CHAIN_CONFIG_ADDRESS)
            .typed(ChainConfigContractProxy)
            .register_bls_key(
                hash_of_hashes,
                ValidatorOperation {
                    validator_data,
                    nonce: operation_nonce,
                },
            )
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run();

        self.assert_expected_error_message(response, None);
        self.assert_expected_log(logs, expected_custom_log, expected_error_log);
    }

    pub fn register_validator_operation(
        &mut self,
        validator_data: ValidatorData<StaticApi>,
        _signature: ManagedBuffer<StaticApi>,
        bitmap: ManagedBuffer<StaticApi>,
        epoch: u64,
    ) {
        let validator_data_hash = validator_data.generate_hash();
        let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&validator_data_hash.to_vec()));

        let (new_signature, pub_keys) = self.get_sig_and_pub_keys(1, &hash_of_hashes);

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(HEADER_VERIFIER_ADDRESS)
            .whitebox(header_verifier::contract_obj, |sc| {
                let pub_key = ManagedBuffer::new_from_bytes(&pub_keys[0].to_vec());
                sc.bls_pub_keys(0).clear();
                sc.bls_pub_keys(0).insert(pub_key);
            });

        self.register_operation(
            OWNER_ADDRESS,
            new_signature,
            &hash_of_hashes,
            bitmap,
            epoch,
            MultiValueEncoded::from_iter(vec![validator_data_hash]),
        );

        let operation_nonce = self.next_operation_nonce();

        self.register_validator(
            &hash_of_hashes,
            validator_data.clone(),
            operation_nonce,
            Some(EXECUTED_BRIDGE_OP_EVENT),
            None,
        );

        assert_eq!(
            self.get_bls_key_id(&validator_data.bls_key),
            validator_data.id
        );
    }

    pub fn unregister_validator(
        &mut self,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        validator_operation: ValidatorOperation<StaticApi>,
        expected_error_message: Option<&str>,
        expected_custom_log: Option<&str>,
    ) {
        let (response, logs) = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(CHAIN_CONFIG_ADDRESS)
            .typed(ChainConfigContractProxy)
            .unregister_bls_key(hash_of_hashes, validator_operation)
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run();

        self.assert_expected_error_message(response, expected_error_message);
        self.assert_expected_log(logs, expected_custom_log, None);
    }

    pub fn set_bls_keys_in_header_storage(&mut self, pub_keys: Vec<ManagedBuffer<StaticApi>>) {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(HEADER_VERIFIER_ADDRESS)
            .whitebox(header_verifier::contract_obj, |sc| {
                let mut new_pub_keys: ManagedVec<
                    VMHooksApi<DebugApiBackend>,
                    ManagedBuffer<VMHooksApi<DebugApiBackend>>,
                > = ManagedVec::new();
                for pub_key in pub_keys.clone() {
                    let pub_key = ManagedBuffer::new_from_bytes(&pub_key.to_vec());
                    new_pub_keys.push(pub_key);
                }
                sc.bls_pub_keys(0).clear();
                sc.bls_pub_keys(0).extend(new_pub_keys);
            });
    }

    pub fn unregister_validator_via_bridge_operation(
        &mut self,
        validator_id: u32,
        validator_bls_key: &ManagedBuffer<StaticApi>,
        num_of_validators: u64,
        bitmap: &ManagedBuffer<StaticApi>,
        epoch: u64,
    ) {
        let validator_data = ValidatorData {
            id: BigUint::from(validator_id),
            address: OWNER_ADDRESS.to_managed_address(),
            bls_key: validator_bls_key.clone(),
        };

        let validator_operation = ValidatorOperation {
            validator_data,
            nonce: self.next_operation_nonce(),
        };

        let validator_data_hash = validator_operation.generate_hash();
        let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&validator_data_hash.to_vec()));
        let (signature, pub_keys) =
            self.get_sig_and_pub_keys(num_of_validators as usize, &hash_of_hashes);

        self.set_bls_keys_in_header_storage(pub_keys);
        self.register_operation(
            OWNER_ADDRESS,
            signature,
            &hash_of_hashes,
            bitmap.clone(),
            epoch,
            MultiValueEncoded::from_iter(vec![validator_data_hash]),
        );

        self.unregister_validator(
            &hash_of_hashes,
            validator_operation,
            None,
            Some(EXECUTED_BRIDGE_OP_EVENT),
        );
    }

    // TODO: Cleanup
    pub fn unregister_validator_operation(
        &mut self,
        validator_data: ValidatorData<StaticApi>,
        _signature: ManagedBuffer<StaticApi>,
        bitmap: ManagedBuffer<StaticApi>,
        epoch: u64,
    ) {
        let validator_data_hash = validator_data.generate_hash();
        let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&validator_data_hash.to_vec()));

        let bitmap_bytes = bitmap.to_vec();
        let signer_count: usize = bitmap_bytes
            .iter()
            .map(|byte| byte.count_ones() as usize)
            .sum();
        let pk_size = signer_count.max(1);

        let (new_signature, pub_keys) = self.get_sig_and_pub_keys(pk_size, &hash_of_hashes);

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(CHAIN_CONFIG_ADDRESS)
            .whitebox(chain_config::contract_obj, |sc| {
                let mut new_pub_keys: ManagedVec<
                    VMHooksApi<DebugApiBackend>,
                    ManagedBuffer<VMHooksApi<DebugApiBackend>>,
                > = ManagedVec::new();
                for pk in pub_keys.clone() {
                    let pk = ManagedBuffer::new_from_bytes(&pk.to_vec());
                    new_pub_keys.push(pk);
                }
                let id = validator_data.id.to_u64().unwrap();
                let target_index = id.saturating_sub(1) as usize;
                if target_index < new_pub_keys.len() {
                    let new_key = new_pub_keys.get(target_index).clone();
                    sc.validator_info(&BigUint::from(id))
                        .update(|v| v.bls_key = new_key);
                }
            });

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(HEADER_VERIFIER_ADDRESS)
            .whitebox(header_verifier::contract_obj, |sc| {
                let mut new_pub_keys: ManagedVec<
                    VMHooksApi<DebugApiBackend>,
                    ManagedBuffer<VMHooksApi<DebugApiBackend>>,
                > = ManagedVec::new();
                for pub_key in pub_keys.clone() {
                    let pub_key = ManagedBuffer::new_from_bytes(&pub_key.to_vec());
                    new_pub_keys.push(pub_key);
                }
                sc.bls_pub_keys(0).clear();
                sc.bls_pub_keys(0).extend(new_pub_keys);
            });

        self.register_operation(
            OWNER_ADDRESS,
            new_signature,
            &hash_of_hashes,
            bitmap,
            epoch,
            MultiValueEncoded::from_iter(vec![validator_data_hash]),
        );

        let operation_nonce = self.next_operation_nonce();

        let validator_operation = ValidatorOperation {
            validator_data: validator_data.clone(),
            nonce: operation_nonce,
        };

        self.unregister_validator(
            &hash_of_hashes,
            validator_operation,
            None,
            Some(EXECUTED_BRIDGE_OP_EVENT),
        );

        assert_eq!(self.get_bls_key_id(&validator_data.bls_key), 0);
    }

    pub fn get_bls_key_id(&mut self, bls_key: &ManagedBuffer<StaticApi>) -> BigUint<StaticApi> {
        self.world
            .query()
            .to(CHAIN_CONFIG_ADDRESS)
            .typed(ChainConfigContractProxy)
            .bls_key_to_id_mapper(bls_key)
            .returns(ReturnsResult)
            .run()
    }
}
