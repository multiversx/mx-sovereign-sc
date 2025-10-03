use common_test_setup::base_setup::helpers::BLSKey;
use common_test_setup::constants::{
    ESDT_SAFE_ADDRESS, EXECUTED_BRIDGE_OP_EVENT, FEE_MARKET_ADDRESS, FIRST_TEST_TOKEN,
    OWNER_ADDRESS, OWNER_BALANCE, SECOND_TEST_TOKEN, USER_ADDRESS, WRONG_TOKEN_ID,
};
use error_messages::{
    CURRENT_OPERATION_NOT_REGISTERED, INVALID_FEE, INVALID_FEE_TYPE, INVALID_TOKEN_ID,
    PAYMENT_DOES_NOT_COVER_FEE, SETUP_PHASE_NOT_COMPLETED, TOKEN_NOT_ACCEPTED_AS_FEE,
};
use fee_common::storage::FeeCommonStorageModule;
use multiversx_sc::types::EgldOrEsdtTokenIdentifier;
use multiversx_sc::{
    imports::OptionalValue,
    types::{BigUint, ManagedBuffer, ManagedVec, MultiEgldOrEsdtPayment, MultiValueEncoded},
};
use multiversx_sc_scenario::{
    api::StaticApi, multiversx_chain_vm::crypto_functions::sha256, ScenarioTxWhitebox,
};
use structs::fee::{RemoveFeeOperation, SetFeeOperation};
use structs::{
    fee::{
        AddUsersToWhitelistOperation, AddressPercentagePair, DistributeFeesOperation, FeeStruct,
        FeeType, RemoveUsersFromWhitelistOperation,
    },
    forge::ScArray,
    generate_hash::GenerateHash,
};

use crate::mvx_fee_market_blackbox_setup::{MvxFeeMarketTestState, WantedFeeType};

mod mvx_fee_market_blackbox_setup;

#[test]
fn test_deploy_fee_market() {
    let mut state = MvxFeeMarketTestState::new();

    let fee = state.get_fee();

    state
        .common_setup
        .deploy_fee_market(Some(fee), ESDT_SAFE_ADDRESS);
}

/// ### TEST
/// F-MARKET_SET_FEE_DURING_SETUP_PHASE_FAIL
///
/// ### ACTION
/// Call 'set_fee_during_setup_phase()' with wrong parameters
///
/// ### EXPECTED
/// Errors: INVALID_TOKEN_ID, INVALID_FEE_TYPE, INVALID_FEE
#[test]
fn test_set_fee_during_setup_phase_wrong_params() {
    let mut state = MvxFeeMarketTestState::new();

    let fee = state.get_fee();

    state
        .common_setup
        .deploy_fee_market(Some(fee), ESDT_SAFE_ADDRESS);

    state.set_fee_during_setup_phase(
        EgldOrEsdtTokenIdentifier::esdt(WRONG_TOKEN_ID),
        WantedFeeType::Fixed,
        Some(INVALID_TOKEN_ID),
    );

    state.set_fee_during_setup_phase(
        EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN),
        WantedFeeType::None,
        Some(INVALID_FEE_TYPE),
    );

    state.set_fee_during_setup_phase(
        EgldOrEsdtTokenIdentifier::esdt(SECOND_TEST_TOKEN),
        WantedFeeType::Fixed,
        Some(INVALID_FEE),
    );
}

/// ### TEST
/// F-MARKET_SET_FEE_FAIL
///
/// ### ACTION
/// Call `set_fee()` when setup phase is not completed
///
/// ### EXPECTED
/// Error CALLER_NOT_OWNER
#[test]
fn test_set_fee_setup_not_completed() {
    let mut state = MvxFeeMarketTestState::new();

    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::FeeMarket, ScArray::ChainConfig]);

    let fee_struct = FeeStruct {
        base_token: EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN),
        fee_type: FeeType::Fixed {
            token: EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN),
            per_transfer: BigUint::default(),
            per_gas: BigUint::default(),
        },
    };

    let set_fee_operation = SetFeeOperation {
        fee_struct,
        nonce: state.common_setup.next_operation_nonce(),
    };

    state.set_fee(
        &ManagedBuffer::new(),
        set_fee_operation,
        Some(EXECUTED_BRIDGE_OP_EVENT),
        Some(SETUP_PHASE_NOT_COMPLETED),
    );
}

/// ### TEST
/// F-MARKET_REMOVE_USERS_FROM_WHITELIST_OK
///
/// ### ACTION
/// Call 'remove_users_from_whitelist`
///
/// ### EXPECTED
/// SC whitelist is updated
#[test]
fn test_remove_users_from_whitelist() {
    let mut state = MvxFeeMarketTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    let new_users = vec![
        USER_ADDRESS.to_managed_address(),
        OWNER_ADDRESS.to_managed_address(),
    ];

    let operation_one = AddUsersToWhitelistOperation {
        nonce: state.common_setup.next_operation_nonce(),
        users: ManagedVec::from_iter(new_users.clone()),
    };
    let operation_two = RemoveUsersFromWhitelistOperation {
        users: ManagedVec::from_iter(new_users.clone()),
        nonce: state.common_setup.next_operation_nonce(),
    };

    let operation_one_hash = operation_one.generate_hash();
    let mut aggregated_hashes = operation_one_hash.clone();
    let operation_two_hash = operation_two.generate_hash();
    aggregated_hashes.append(&operation_two_hash);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&aggregated_hashes.to_vec()));

    let (signature, public_keys) = state.common_setup.get_sig_and_pub_keys(1, &hash_of_hashes);

    state.common_setup.register(
        public_keys.first().unwrap(),
        &MultiEgldOrEsdtPayment::new(),
        None,
    );

    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::FeeMarket, ScArray::ChainConfig]);

    state.common_setup.complete_fee_market_setup_phase();

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let bitmap = ManagedBuffer::new_from_bytes(&[0x01]);
    let epoch = 0;

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        signature,
        &hash_of_hashes,
        bitmap,
        epoch,
        MultiValueEncoded::from_iter(vec![operation_one_hash, operation_two_hash]),
    );
    state.add_users_to_whitelist(&hash_of_hashes, operation_one);

    state
        .common_setup
        .query_user_fee_whitelist(Some(&new_users));

    state.remove_users_from_whitelist(&hash_of_hashes, operation_two);

    state
        .common_setup
        .query_user_fee_whitelist(Some(&new_users));
}

/// ### TEST
/// F-MARKET_SET_FEE_OK
///
/// ### ACTION
/// Call `set_fee()`
///
/// ### EXPECTED
/// Fee is set in contract's storage
#[test]
fn test_set_fee() {
    let mut state = MvxFeeMarketTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    let fee_struct = FeeStruct {
        base_token: EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN),
        fee_type: FeeType::Fixed {
            token: EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN),
            per_transfer: BigUint::default(),
            per_gas: BigUint::default(),
        },
    };

    let set_fee_operation = SetFeeOperation {
        fee_struct,
        nonce: state.common_setup.next_operation_nonce(),
    };
    let fee_hash = set_fee_operation.generate_hash();
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&fee_hash.to_vec()));

    let (signature, public_keys) = state.common_setup.get_sig_and_pub_keys(1, &hash_of_hashes);

    state.common_setup.register(
        public_keys.first().unwrap(),
        &MultiEgldOrEsdtPayment::new(),
        None,
    );

    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::FeeMarket, ScArray::ChainConfig]);

    state.common_setup.complete_fee_market_setup_phase();

    let bitmap = ManagedBuffer::new_from_bytes(&[0x01]);
    let epoch = 0;

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        signature,
        &hash_of_hashes,
        bitmap,
        epoch,
        MultiValueEncoded::from_iter(vec![fee_hash]),
    );

    state.set_fee(
        &hash_of_hashes,
        set_fee_operation,
        Some(EXECUTED_BRIDGE_OP_EVENT),
        None,
    );

    state
        .common_setup
        .world
        .query()
        .to(FEE_MARKET_ADDRESS)
        .whitebox(mvx_fee_market::contract_obj, |sc| {
            assert!(!sc
                .token_fee(&EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN))
                .is_empty());
        });
}

/// ### TEST
/// F-MARKET_REMOVE_FEE_FAIL
///
/// ### ACTION
/// Call `remove_fee()` when setup was not completed
///
/// ### EXPECTED
/// Error CALLER_NOT_OWNER
#[test]
fn test_remove_fee_setup_phase_not_completed() {
    let mut state = MvxFeeMarketTestState::new();

    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::FeeMarket]);

    let remove_fee_operation = RemoveFeeOperation {
        token_id: EgldOrEsdtTokenIdentifier::from(FIRST_TEST_TOKEN.as_str()),
        nonce: state.common_setup.next_operation_nonce(),
    };

    state.remove_fee(
        &ManagedBuffer::new(),
        remove_fee_operation,
        None,
        Some(EXECUTED_BRIDGE_OP_EVENT),
        Some(SETUP_PHASE_NOT_COMPLETED),
    );
}

/// ### TEST
/// F-MARKET_REMOVE_FEE_OK
///
/// ### ACTION
/// Register `set_fee()` and `remove_fee()` separately and then call `remove_fee`
///
/// ### EXPECTED
/// Fee is removed the contract's storage
#[test]
fn test_remove_fee_register_separate_operations() {
    let mut state = MvxFeeMarketTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    let fee_struct = FeeStruct {
        base_token: EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN),
        fee_type: FeeType::Fixed {
            token: EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN),
            per_transfer: BigUint::default(),
            per_gas: BigUint::default(),
        },
    };
    let set_fee_operation = SetFeeOperation {
        fee_struct,
        nonce: state.common_setup.next_operation_nonce(),
    };
    let register_fee_hash = set_fee_operation.generate_hash();
    let register_fee_hash_of_hashes =
        ManagedBuffer::new_from_bytes(&sha256(&register_fee_hash.to_vec()));

    let (signature, public_keys) = state
        .common_setup
        .get_sig_and_pub_keys(1, &register_fee_hash_of_hashes);

    state.common_setup.register(
        public_keys.first().unwrap(),
        &MultiEgldOrEsdtPayment::new(),
        None,
    );

    let remove_fee_operation = RemoveFeeOperation {
        token_id: EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN),
        nonce: state.common_setup.next_operation_nonce(),
    };
    let remove_fee_hash = remove_fee_operation.generate_hash();
    let remove_fee_hash_of_hashes =
        ManagedBuffer::new_from_bytes(&sha256(&remove_fee_hash.to_vec()));

    let (signature_remove_fee, public_keys_remove_fee) = state
        .common_setup
        .get_sig_and_pub_keys(1, &remove_fee_hash_of_hashes);

    state.common_setup.register(
        public_keys_remove_fee.first().unwrap(),
        &MultiEgldOrEsdtPayment::new(),
        None,
    );

    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig, ScArray::FeeMarket]);

    state.common_setup.complete_fee_market_setup_phase();

    let bitmap = ManagedBuffer::new_from_bytes(&[0x01]);
    let epoch = 0;

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        signature,
        &register_fee_hash_of_hashes,
        bitmap,
        epoch,
        MultiValueEncoded::from_iter(vec![register_fee_hash]),
    );

    state.set_fee(
        &register_fee_hash_of_hashes,
        set_fee_operation,
        Some(EXECUTED_BRIDGE_OP_EVENT),
        None,
    );

    state
        .common_setup
        .world
        .query()
        .to(FEE_MARKET_ADDRESS)
        .whitebox(mvx_fee_market::contract_obj, |sc| {
            assert!(!sc
                .token_fee(&EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN))
                .is_empty());
        });

    let bitmap = ManagedBuffer::new_from_bytes(&[0x02]);
    let epoch = 0;

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        signature_remove_fee,
        &remove_fee_hash_of_hashes,
        bitmap,
        epoch,
        MultiValueEncoded::from_iter(vec![remove_fee_hash]),
    );

    state.remove_fee(
        &remove_fee_hash_of_hashes,
        remove_fee_operation,
        None,
        Some(EXECUTED_BRIDGE_OP_EVENT),
        None,
    );

    state
        .common_setup
        .world
        .query()
        .to(FEE_MARKET_ADDRESS)
        .whitebox(mvx_fee_market::contract_obj, |sc| {
            assert!(sc
                .token_fee(&EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN))
                .is_empty());
        });
}

/// ### TEST
/// F-MARKET_REMOVE_FEE_OK
///
/// ### ACTION
/// Register both `set_fee()` and `remove_fee()` at the same time and then call `remove_fee`
///
/// ### EXPECTED
/// Fee is removed the contract's storage
#[test]
fn test_remove_fee_register_with_one_hash_of_hashes() {
    let mut state = MvxFeeMarketTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    let fee_struct = FeeStruct {
        base_token: EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN),
        fee_type: FeeType::Fixed {
            token: EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN),
            per_transfer: BigUint::default(),
            per_gas: BigUint::default(),
        },
    };

    let set_fee_operation = SetFeeOperation {
        fee_struct,
        nonce: state.common_setup.next_operation_nonce(),
    };

    let remove_fee_operation = RemoveFeeOperation {
        token_id: EgldOrEsdtTokenIdentifier::from(FIRST_TEST_TOKEN.as_str()),
        nonce: state.common_setup.next_operation_nonce(),
    };

    let remove_fee_hash: ManagedBuffer<StaticApi> = remove_fee_operation.generate_hash();
    let register_fee_hash = set_fee_operation.generate_hash();
    let mut aggregated_hashes = ManagedBuffer::new();

    aggregated_hashes.append(&remove_fee_hash);
    aggregated_hashes.append(&register_fee_hash);

    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&aggregated_hashes.to_vec()));

    let (signature, public_keys) = state.common_setup.get_sig_and_pub_keys(1, &hash_of_hashes);

    state.common_setup.register(
        public_keys.first().unwrap(),
        &MultiEgldOrEsdtPayment::new(),
        None,
    );

    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig, ScArray::FeeMarket]);

    state.common_setup.complete_fee_market_setup_phase();

    let bitmap = ManagedBuffer::new_from_bytes(&[0x01]);
    let epoch = 0;

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        signature,
        &hash_of_hashes,
        bitmap,
        epoch,
        MultiValueEncoded::from_iter(vec![remove_fee_hash, register_fee_hash]),
    );

    state.set_fee(
        &hash_of_hashes,
        set_fee_operation,
        Some(EXECUTED_BRIDGE_OP_EVENT),
        None,
    );

    state
        .common_setup
        .world
        .query()
        .to(FEE_MARKET_ADDRESS)
        .whitebox(mvx_fee_market::contract_obj, |sc| {
            assert!(!sc
                .token_fee(&EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN))
                .is_empty());
        });

    state.remove_fee(
        &hash_of_hashes,
        remove_fee_operation,
        None,
        Some(EXECUTED_BRIDGE_OP_EVENT),
        None,
    );

    state
        .common_setup
        .world
        .query()
        .to(FEE_MARKET_ADDRESS)
        .whitebox(mvx_fee_market::contract_obj, |sc| {
            assert!(sc
                .token_fee(&EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN))
                .is_empty());
        });
}

/// ### TEST
/// F-MARKET_DISTRIBUTE_FEES_FAIL
///
/// ### ACTION
/// Call 'distribute_fees()' when setup is not completed
///
/// ### EXPECTED
/// Error CALLER_NOT_OWNER
#[test]
fn distribute_fees_setup_not_completed() {
    let mut state = MvxFeeMarketTestState::new();

    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::FeeMarket]);

    let operation_nonce = state.common_setup.next_operation_nonce();
    let operation = DistributeFeesOperation {
        pairs: ManagedVec::new(),
        nonce: operation_nonce,
    };
    state.distribute_fees(
        &ManagedBuffer::new(),
        operation,
        Some(EXECUTED_BRIDGE_OP_EVENT),
        Some(SETUP_PHASE_NOT_COMPLETED),
    );
}

/// ### TEST
/// F-MARKET_DISTRIBUTE_FEES_FAIL
///
/// ### ACTION
/// Call 'distribute_fees()' when operation is not registered
///
/// ### EXPECTED
/// Error CURRENT_OPERATION_NOT_REGISTERED
#[test]
fn distribute_fees_operation_not_registered() {
    let mut state = MvxFeeMarketTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    state
        .common_setup
        .register(&BLSKey::random(), &MultiEgldOrEsdtPayment::new(), None);

    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);

    state.common_setup.complete_fee_market_setup_phase();

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig, ScArray::FeeMarket]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let operation_nonce = state.common_setup.next_operation_nonce();
    let operation = DistributeFeesOperation {
        pairs: ManagedVec::new(),
        nonce: operation_nonce,
    };
    state.distribute_fees(
        &ManagedBuffer::new(),
        operation,
        Some(EXECUTED_BRIDGE_OP_EVENT),
        Some(CURRENT_OPERATION_NOT_REGISTERED),
    );
}

/// ### TEST
/// F-MARKET_DISTRIBUTE_FEES_FAIL
///
/// ### ACTION
/// Call 'distribute_fees()' with one pair
///
/// ### EXPECTED
/// OWNER balance is unchanged, `failedBridgeOp` event emitted
#[test]
fn test_distribute_fees_percentage_under_limit() {
    let mut state = MvxFeeMarketTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    let address_pair: AddressPercentagePair<StaticApi> = AddressPercentagePair {
        address: OWNER_ADDRESS.to_managed_address(),
        percentage: 10,
    };

    let operation_nonce = state.common_setup.next_operation_nonce();
    let operation = DistributeFeesOperation {
        pairs: ManagedVec::from_iter(vec![address_pair.clone()]),
        nonce: operation_nonce,
    };

    let operation_hash = operation.generate_hash();
    let mut aggregated_hash: ManagedBuffer<StaticApi> = ManagedBuffer::new();
    aggregated_hash.append(&operation_hash);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&aggregated_hash.to_vec()));

    let (signature, public_keys) = state.common_setup.get_sig_and_pub_keys(1, &hash_of_hashes);

    state.common_setup.register(
        public_keys.first().unwrap(),
        &MultiEgldOrEsdtPayment::new(),
        None,
    );

    state.common_setup.complete_chain_config_setup_phase();

    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);

    state.common_setup.complete_fee_market_setup_phase();

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig, ScArray::FeeMarket]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let bitmap = ManagedBuffer::new_from_bytes(&[0x01]);
    let epoch = 0;

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        signature,
        &hash_of_hashes,
        bitmap,
        epoch,
        MultiValueEncoded::from_iter(vec![operation_hash]),
    );

    state.distribute_fees(
        &hash_of_hashes,
        operation,
        Some(EXECUTED_BRIDGE_OP_EVENT),
        None,
    );
}

/// ### TEST
/// F-MARKET_DISTRIBUTE_FEES_OK
///
/// ### ACTION
/// Call 'distribute_fees()' with one pair
///
/// ### EXPECTED
/// OWNER balance is changed, `executedBridgeOp` event emitted
#[test]
fn test_distribute_fees() {
    let mut state = MvxFeeMarketTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    let address_pair: AddressPercentagePair<StaticApi> = AddressPercentagePair {
        address: OWNER_ADDRESS.to_managed_address(),
        percentage: 10_000,
    };

    let operation = DistributeFeesOperation {
        pairs: ManagedVec::from_iter(vec![address_pair.clone()]),
        nonce: state.common_setup.next_operation_nonce(),
    };
    let operation_hash = operation.generate_hash();

    let mut aggregated_hash: ManagedBuffer<StaticApi> = ManagedBuffer::new();
    aggregated_hash.append(&operation_hash);

    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&aggregated_hash.to_vec()));

    let (signature, public_keys) = state.common_setup.get_sig_and_pub_keys(1, &hash_of_hashes);
    state.common_setup.register(
        public_keys.first().unwrap(),
        &MultiEgldOrEsdtPayment::new(),
        None,
    );

    state.common_setup.complete_chain_config_setup_phase();

    let fee_per_transfer = BigUint::from(100u32);

    let fee = FeeStruct {
        base_token: EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN),
        fee_type: FeeType::Fixed {
            token: EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN),
            per_transfer: fee_per_transfer.clone(),
            per_gas: BigUint::default(),
        },
    };

    state
        .common_setup
        .deploy_fee_market(Some(fee), ESDT_SAFE_ADDRESS);

    state.subtract_fee(
        WantedFeeType::Correct,
        USER_ADDRESS.to_address(),
        1u64 as usize,
        OptionalValue::Some(30u64),
        None,
    );

    state.common_setup.complete_fee_market_setup_phase();

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig, ScArray::FeeMarket]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let bitmap = ManagedBuffer::new_from_bytes(&[0x01]);
    let epoch = 0;

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        signature,
        &hash_of_hashes,
        bitmap,
        epoch,
        MultiValueEncoded::from_iter(vec![operation_hash]),
    );

    state.distribute_fees(
        &hash_of_hashes,
        operation,
        Some(EXECUTED_BRIDGE_OP_EVENT),
        None,
    );

    state.common_setup.check_account_single_esdt(
        OWNER_ADDRESS.to_address(),
        FIRST_TEST_TOKEN,
        0,
        BigUint::from(OWNER_BALANCE) + fee_per_transfer,
    );
}

/// ### TEST
/// F-MARKET_SUBTRACT_FEE_OK
///
/// ### ACTION
/// Call 'subtract_fee()' with no fee set
///
/// ### EXPECTED
/// User balance is unchanged
#[test]
fn test_subtract_fee_no_fee() {
    let mut state = MvxFeeMarketTestState::new();

    let fee = state.get_fee();

    state
        .common_setup
        .deploy_fee_market(Some(fee), ESDT_SAFE_ADDRESS);

    state.remove_fee_during_setup_phase(FIRST_TEST_TOKEN);

    state.subtract_fee(
        WantedFeeType::Correct,
        USER_ADDRESS.to_address(),
        1u64 as usize,
        OptionalValue::Some(30u64),
        None,
    );

    state.common_setup.check_account_single_esdt(
        ESDT_SAFE_ADDRESS.to_address(),
        FIRST_TEST_TOKEN,
        0u64,
        BigUint::from(OWNER_BALANCE),
    );

    state.common_setup.check_account_single_esdt(
        USER_ADDRESS.to_address(),
        FIRST_TEST_TOKEN,
        0u64,
        BigUint::from(OWNER_BALANCE),
    );
}

/// ### TEST
/// F-MARKET_SUBTRACT_FEE_OK
///
/// ### ACTION
/// Call 'subtract_fee()' with a whitelisted user
///
/// ### EXPECTED
/// User balance is unchanged
#[test]
fn test_subtract_fee_whitelisted() {
    let mut state = MvxFeeMarketTestState::new();

    let fee = state.get_fee();

    state
        .common_setup
        .deploy_fee_market(Some(fee), ESDT_SAFE_ADDRESS);

    let whitelisted_users = vec![USER_ADDRESS];

    state.add_users_to_whitelist_during_setup_phase(whitelisted_users);

    state.subtract_fee(
        WantedFeeType::Correct,
        USER_ADDRESS.to_address(),
        1u64 as usize,
        OptionalValue::Some(30u64),
        None,
    );

    state.common_setup.check_account_single_esdt(
        ESDT_SAFE_ADDRESS.to_address(),
        FIRST_TEST_TOKEN,
        0u64,
        BigUint::from(OWNER_BALANCE),
    );

    state.common_setup.check_account_single_esdt(
        USER_ADDRESS.to_address(),
        FIRST_TEST_TOKEN,
        0u64,
        BigUint::from(OWNER_BALANCE),
    );
}

/// ### TEST
/// F-MARKET_SUBTRACT_FEE_FAIL
///
/// ### ACTION
/// Call 'subtract_fee()' with an invalid payment token
///
/// ### EXPECTED
/// Error TOKEN_NOT_ACCEPTED_AS_FEE
#[test]
fn test_subtract_fee_invalid_payment_token() {
    let mut state = MvxFeeMarketTestState::new();

    let fee = state.get_fee();

    state
        .common_setup
        .deploy_fee_market(Some(fee), ESDT_SAFE_ADDRESS);

    state.subtract_fee(
        WantedFeeType::InvalidToken,
        USER_ADDRESS.to_address(),
        1u64 as usize,
        OptionalValue::Some(30u64),
        Some(TOKEN_NOT_ACCEPTED_AS_FEE),
    );

    state.common_setup.check_account_single_esdt(
        ESDT_SAFE_ADDRESS.to_address(),
        FIRST_TEST_TOKEN,
        0u64,
        BigUint::from(OWNER_BALANCE),
    );

    state.common_setup.check_account_single_esdt(
        USER_ADDRESS.to_address(),
        FIRST_TEST_TOKEN,
        0u64,
        BigUint::from(OWNER_BALANCE),
    );
}

/// ### TEST
/// F-MARKET_SUBTRACT_FEE_FAIL
///
/// ### ACTION
/// Call 'subtract_fee()' with not enough tokens to cover the fee
///
/// ### EXPECTED
/// Error PAYMENT_DOES_NOT_COVER_FEE
#[test]
fn test_subtract_fixed_fee_payment_not_covered() {
    let mut state = MvxFeeMarketTestState::new();

    let fee = state.get_fee();

    state
        .common_setup
        .deploy_fee_market(Some(fee), ESDT_SAFE_ADDRESS);
    state
        .common_setup
        .change_ownership_to_header_verifier(FEE_MARKET_ADDRESS);

    state.subtract_fee(
        WantedFeeType::LessThanFee,
        USER_ADDRESS.to_address(),
        1u64 as usize,
        OptionalValue::Some(30u64),
        Some(PAYMENT_DOES_NOT_COVER_FEE),
    );

    state.common_setup.check_account_single_esdt(
        ESDT_SAFE_ADDRESS.to_address(),
        FIRST_TEST_TOKEN,
        0u64,
        BigUint::from(OWNER_BALANCE),
    );

    state.common_setup.check_account_single_esdt(
        USER_ADDRESS.to_address(),
        FIRST_TEST_TOKEN,
        0u64,
        BigUint::from(OWNER_BALANCE),
    );
}

/// ### TEST
/// F-MARKET_SUBTRACT_FEE_OK
///
/// ### ACTION
/// Call 'subtract_fee()' with payment bigger than fee
///
/// ### EXPECTED
/// User balance is refunded with the difference
#[test]
fn test_subtract_fee_fixed_payment_bigger_than_fee() {
    let mut state = MvxFeeMarketTestState::new();

    let fee = state.get_fee();

    state
        .common_setup
        .deploy_fee_market(Some(fee), ESDT_SAFE_ADDRESS);
    state
        .common_setup
        .change_ownership_to_header_verifier(FEE_MARKET_ADDRESS);

    state.subtract_fee(
        WantedFeeType::Correct,
        USER_ADDRESS.to_address(),
        1u64 as usize,
        OptionalValue::Some(30u64),
        None,
    );

    state.common_setup.check_account_single_esdt(
        ESDT_SAFE_ADDRESS.to_address(),
        FIRST_TEST_TOKEN,
        0u64,
        BigUint::from(OWNER_BALANCE - 200),
    );

    state.common_setup.check_account_single_esdt(
        USER_ADDRESS.to_address(),
        FIRST_TEST_TOKEN,
        0u64,
        BigUint::from(OWNER_BALANCE + 100),
    );
}
