use cross_chain::storage::CrossChainStorage;
use header_verifier::storage::HeaderVerifierStorageModule;
use multiversx_sc_scenario::imports::{EgldOrEsdtTokenIdentifier, ManagedVec};
use multiversx_sc_scenario::DebugApi;
use multiversx_sc_scenario::{
    api::StaticApi,
    imports::{
        Address, BigUint, ManagedAddress, ManagedBuffer, MultiValue3, ReturnsResultUnmanaged,
        TestTokenIdentifier,
    },
    multiversx_chain_vm::crypto_functions::sha256,
    scenario_model::{Log, TxResponseStatus},
    ScenarioTxRun, ScenarioTxWhitebox,
};
use mvx_esdt_safe::bridging_mechanism::BridgingMechanism;
use proxies::mvx_fee_market_proxy::MvxFeeMarketProxy;
use structs::OperationHashStatus;

use crate::{
    base_setup::init::BaseSetup,
    constants::{
        CHAIN_CONFIG_ADDRESS, ESDT_SAFE_ADDRESS, FEE_MARKET_ADDRESS, FIRST_TEST_TOKEN,
        HEADER_VERIFIER_ADDRESS, OWNER_ADDRESS,
    },
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

    pub fn query_user_fee_whitelist(
        &mut self,
        users_to_query: Option<&[ManagedAddress<StaticApi>]>,
    ) {
        let query = self
            .world
            .query()
            .to(FEE_MARKET_ADDRESS)
            .typed(MvxFeeMarketProxy)
            .users_whitelist()
            .returns(ReturnsResultUnmanaged)
            .run();

        match users_to_query {
            Some(expected_users) => {
                assert!(query
                    .iter()
                    .all(|u| expected_users.contains(&ManagedAddress::from(u))))
            }
            None => {
                assert!(query.is_empty())
            }
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

    pub fn check_bls_key_for_epoch_in_header_verifier(
        &mut self,
        epoch: u64,
        registered_bls_keys: &ManagedVec<StaticApi, ManagedBuffer<StaticApi>>,
    ) {
        // Convert ManagedVec<...> -> Vec<String> (hex encoded)
        let bls_keys_hex: Vec<String> = registered_bls_keys
            .iter()
            .map(|buffer| {
                let bytes = buffer.to_boxed_bytes();
                hex::encode(bytes) // encode each buffer as a hex string
            })
            .collect();

        // Borrow as &str for iteration
        let bls_keys: Vec<&str> = bls_keys_hex.iter().map(|s| s.as_str()).collect();

        // Query contract and assert
        self.world.query().to(HEADER_VERIFIER_ADDRESS).whitebox(
            header_verifier::contract_obj,
            |sc| {
                for bls_key_hex in bls_keys {
                    // Convert hex string back to bytes and build ManagedBuffer<DebugApi>
                    let key_bytes = hex::decode(bls_key_hex).unwrap();
                    let buffer = ManagedBuffer::new_from_bytes(&key_bytes);

                    assert!(
                        sc.bls_pub_keys(epoch).contains(&buffer),
                        "BLS key not found in header verifier: {}",
                        bls_key_hex
                    );
                }
            },
        );
    }

    pub fn check_deposited_tokens_amount(
        &mut self,
        tokens: Vec<(EgldOrEsdtTokenIdentifier<StaticApi>, u64)>,
    ) {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .whitebox(mvx_esdt_safe::contract_obj, |sc| {
                let tokens: Vec<(EgldOrEsdtTokenIdentifier<DebugApi>, u64)> = tokens
                    .into_iter()
                    .map(|(token_id, amount)| {
                        let token_id_bytes = token_id.to_boxed_bytes();
                        (
                            EgldOrEsdtTokenIdentifier::<DebugApi>::from(token_id_bytes.as_slice()),
                            amount,
                        )
                    })
                    .collect();
                for token in tokens {
                    let (token_id, amount) = token;
                    if amount == 0 {
                        let stored_amount = sc.deposited_tokens_amount(&token_id).get();
                        assert!(
                            sc.deposited_tokens_amount(&token_id).is_empty(),
                            "Expected no storage entry for token {:?}, but found: {:?}",
                            token_id,
                            stored_amount
                        );
                    } else {
                        assert!(
                            sc.deposited_tokens_amount(&token_id).get() == amount,
                            "Expected deposited amount for token {:?} to be {:?}, but found {:?}",
                            token_id,
                            amount,
                            sc.deposited_tokens_amount(&token_id).get()
                        );
                    }
                }
            });
    }

    pub fn check_multiversx_to_sovereign_token_id_mapper_is_empty(&mut self, token_name: &str) {
        self.world
            .query()
            .to(ESDT_SAFE_ADDRESS)
            .whitebox(mvx_esdt_safe::contract_obj, |sc| {
                assert!(sc
                    .multiversx_to_sovereign_token_id_mapper(&EgldOrEsdtTokenIdentifier::from(
                        token_name
                    ))
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
    //TODO: Remove the empty string check after callback fix in blackbox
    pub fn assert_expected_log(
        &mut self,
        logs: Vec<Log>,
        expected_log: Option<&str>,
        expected_log_error: Option<&str>,
    ) {
        match expected_log {
            None => {
                assert!(
                    logs.is_empty(),
                    "Expected no logs, but found some: {:?}",
                    logs
                );

                assert!(
                    expected_log_error.is_none(),
                    "Expected no logs, but wanted to check for error: {}",
                    expected_log_error.unwrap()
                );
            }
            Some(expected_str) => {
                // assert!(!expected_str.is_empty(), "{}", EMPTY_EXPECTED_LOG);
                if expected_str.is_empty() {
                    return;
                }
                let expected_bytes = ManagedBuffer::<StaticApi>::from(expected_str).to_vec();

                let matching_logs: Vec<&Log> = logs
                    .iter()
                    .filter(|log| {
                        let topic_match = log.topics.iter().any(|topic| {
                            topic
                                .windows(expected_bytes.len())
                                .any(|window| window == expected_bytes)
                        });
                        let data_match = log.data.iter().any(|data_item| {
                            data_item
                                .to_vec()
                                .windows(expected_bytes.len())
                                .any(|window| window == expected_bytes)
                        });
                        topic_match || data_match
                    })
                    .collect();

                assert!(
                    !matching_logs.is_empty(),
                    "Expected log '{}' not found",
                    expected_str
                );

                if let Some(expected_error) = expected_log_error {
                    let expected_error_bytes =
                        ManagedBuffer::<StaticApi>::from(expected_error).to_vec();

                    let found_error_in_data = matching_logs.iter().any(|log| {
                        log.data.iter().any(|data_item| {
                            let v = data_item.to_vec();
                            v.windows(expected_error_bytes.len())
                                .any(|w| w == expected_error_bytes)
                        })
                    });

                    assert!(
                        found_error_in_data,
                        "Expected error '{}' not found in data field of any log with topic '{}'",
                        expected_error, expected_str
                    );
                }
            }
        }
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

    pub fn assert_contract_and_owner_balances(
        &mut self,
        contract_egld: &BigUint<StaticApi>,
        contract_token: &BigUint<StaticApi>,
        owner_egld: &BigUint<StaticApi>,
        owner_token: &BigUint<StaticApi>,
    ) {
        self.world
            .check_account(CHAIN_CONFIG_ADDRESS)
            .balance(contract_egld);
        self.world
            .check_account(CHAIN_CONFIG_ADDRESS)
            .esdt_balance(FIRST_TEST_TOKEN, contract_token);
        self.world.check_account(OWNER_ADDRESS).balance(owner_egld);
        self.world
            .check_account(OWNER_ADDRESS)
            .esdt_balance(FIRST_TEST_TOKEN, owner_token);
    }

    pub fn assert_contract_and_owner_token_balances(
        &mut self,
        contract_token: &BigUint<StaticApi>,
        owner_token: &BigUint<StaticApi>,
    ) {
        self.world
            .check_account(CHAIN_CONFIG_ADDRESS)
            .esdt_balance(FIRST_TEST_TOKEN, contract_token);
        self.world
            .check_account(OWNER_ADDRESS)
            .esdt_balance(FIRST_TEST_TOKEN, owner_token);
    }
}
