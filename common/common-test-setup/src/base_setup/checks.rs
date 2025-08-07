use cross_chain::storage::CrossChainStorage;
use error_messages::EMPTY_EXPECTED_LOG;
use header_verifier::{Headerverifier, OperationHashStatus};
use multiversx_sc_scenario::{
    api::StaticApi,
    imports::{Address, BigUint, ManagedBuffer, MultiValue3, TestTokenIdentifier},
    multiversx_chain_vm::crypto_functions::sha256,
    scenario_model::{Log, TxResponseStatus},
    ScenarioTxWhitebox,
};
use mvx_esdt_safe::bridging_mechanism::BridgingMechanism;

use crate::{
    base_setup::init::BaseSetup,
    constants::{ESDT_SAFE_ADDRESS, HEADER_VERIFIER_ADDRESS, OWNER_ADDRESS},
};

impl BaseSetup {
    pub fn check_account_multiple_esdts(
        &mut self,
        address: Address,
        tokens: Vec<MultiValue3<TestTokenIdentifier, u64, BigUint<StaticApi>>>,
    ) {
        for token in tokens {
            let (token_id, nonce, amount) = token.into_tuple();
            self.world
                .check_account(&address)
                .esdt_nft_balance_and_attributes(
                    token_id,
                    nonce,
                    amount,
                    ManagedBuffer::<StaticApi>::new(),
                );
        }
    }

    pub fn check_account_single_esdt(
        &mut self,
        address: Address,
        token_id: TestTokenIdentifier,
        nonce: u64,
        expected_balance: BigUint<StaticApi>,
    ) {
        self.world
            .check_account(address)
            .esdt_nft_balance_and_attributes(
                token_id,
                nonce,
                expected_balance,
                ManagedBuffer::<StaticApi>::new(),
            );
    }

    pub fn check_registered_validator_in_header_verifier(
        &mut self,
        epoch: u64,
        bls_keys: Vec<&str>,
    ) {
        self.world.query().to(HEADER_VERIFIER_ADDRESS).whitebox(
            header_verifier::contract_obj,
            |sc| {
                for bls_key in bls_keys {
                    assert!(sc
                        .bls_pub_keys(epoch)
                        .contains(&ManagedBuffer::from(bls_key)));
                }
            },
        )
    }

    pub fn check_deposited_tokens_amount(&mut self, tokens: Vec<(TestTokenIdentifier, u64)>) {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .whitebox(mvx_esdt_safe::contract_obj, |sc| {
                for token in tokens {
                    let (token_id, amount) = token;
                    assert!(sc.deposited_tokens_amount(&token_id.into()).get() == amount);
                }
            });
    }

    pub fn check_multiversx_to_sovereign_token_id_mapper_is_empty(&mut self, token_name: &str) {
        self.world
            .query()
            .to(ESDT_SAFE_ADDRESS)
            .whitebox(mvx_esdt_safe::contract_obj, |sc| {
                assert!(sc
                    .multiversx_to_sovereign_token_id_mapper(
                        &TestTokenIdentifier::new(token_name).into()
                    )
                    .is_empty());
            });
    }

    pub fn check_operation_hash_status_is_empty(
        &mut self,
        operation_hash: &ManagedBuffer<StaticApi>,
    ) {
        self.world.query().to(HEADER_VERIFIER_ADDRESS).whitebox(
            header_verifier::contract_obj,
            |sc| {
                let operation_hash_whitebox =
                    ManagedBuffer::new_from_bytes(&operation_hash.to_vec());
                let hash_of_hashes =
                    ManagedBuffer::new_from_bytes(&sha256(&operation_hash_whitebox.to_vec()));

                assert!(sc
                    .operation_hash_status(&hash_of_hashes, &operation_hash_whitebox)
                    .is_empty());
            },
        )
    }

    pub fn check_operation_hash_status(
        &mut self,
        operation_hash: &ManagedBuffer<StaticApi>,
        status: OperationHashStatus,
    ) {
        self.world.query().to(HEADER_VERIFIER_ADDRESS).whitebox(
            header_verifier::contract_obj,
            |sc| {
                let operation_hash_whitebox =
                    ManagedBuffer::new_from_bytes(&operation_hash.to_vec());
                let hash_of_hashes =
                    ManagedBuffer::new_from_bytes(&sha256(&operation_hash_whitebox.to_vec()));

                assert!(
                    sc.operation_hash_status(&hash_of_hashes, &operation_hash_whitebox)
                        .get()
                        == status
                );
            },
        )
    }

    //NOTE: transferValue returns an empty log and calling this function on it will panic
    pub fn assert_expected_log(&mut self, logs: Vec<Log>, expected_log: Option<&str>) {
        match expected_log {
            None => {
                assert!(
                    logs.is_empty(),
                    "Expected no logs, but found some: {:?}",
                    logs
                );
            }
            Some(expected_str) => {
                assert!(!expected_str.is_empty(), "{}", EMPTY_EXPECTED_LOG);
                let expected_bytes = ManagedBuffer::<StaticApi>::from(expected_str).to_vec();

                let found_log = logs
                    .iter()
                    .find(|log| log.topics.iter().any(|topic| *topic == expected_bytes));

                assert!(
                    found_log.is_some(),
                    "Expected log '{}' not found",
                    expected_str
                );
            }
        }
    }

    pub fn assert_expected_data(&self, logs: Vec<Log>, expected_data: &str) {
        let expected_bytes = ManagedBuffer::<StaticApi>::from(expected_data).to_vec();

        let found = logs.iter().any(|log| {
            log.data
                .iter()
                .any(|data_item| data_item.to_vec() == expected_bytes)
        });

        assert!(found, "Expected data '{}' not found", expected_data);
    }

    pub fn assert_expected_error_message(
        &mut self,
        response: Result<(), TxResponseStatus>,
        expected_error_message: Option<&str>,
    ) {
        match response {
            Ok(_) => assert!(
                expected_error_message.is_none(),
                "Transaction was successful, but expected error"
            ),
            Err(error) => {
                assert_eq!(expected_error_message, Some(error.message.as_str()))
            }
        }
    }
}
