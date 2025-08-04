use common_test_setup::constants::{
    ESDT_SAFE_ADDRESS, FEE_MARKET_ADDRESS, FIRST_TEST_TOKEN, OWNER_ADDRESS, OWNER_BALANCE,
    SECOND_TEST_TOKEN, USER_ADDRESS, WRONG_TOKEN_ID,
};
use error_messages::{
    CALLER_NOT_OWNER, CURRENT_OPERATION_NOT_REGISTERED, INVALID_FEE, INVALID_FEE_TYPE,
    INVALID_TOKEN_ID, PAYMENT_DOES_NOT_COVER_FEE, SETUP_PHASE_NOT_COMPLETED,
    TOKEN_NOT_ACCEPTED_AS_FEE,
};
use fee_market::fee_type::FeeTypeModule;
use fee_market_blackbox_setup::*;
use multiversx_sc::{
    imports::{MultiValue2, OptionalValue},
    types::{BigUint, ManagedBuffer, MultiValueEncoded},
};
use multiversx_sc_scenario::{
    api::StaticApi, multiversx_chain_vm::crypto_functions::sha256, ScenarioTxWhitebox,
};
use structs::{
    fee::{AddressPercentagePair, FeeStruct, FeeType},
    forge::ScArray,
    generate_hash::GenerateHash,
};

mod fee_market_blackbox_setup;

#[test]
fn test_deploy_fee_market() {
    let mut state = FeeMarketTestState::new();

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
    let mut state = FeeMarketTestState::new();

    let fee = state.get_fee();

    state
        .common_setup
        .deploy_fee_market(Some(fee), ESDT_SAFE_ADDRESS);

    state.set_fee_during_setup_phase(WRONG_TOKEN_ID, WantedFeeType::Fixed, Some(INVALID_TOKEN_ID));

    state.set_fee_during_setup_phase(
        FIRST_TEST_TOKEN,
        WantedFeeType::None,
        Some(INVALID_FEE_TYPE),
    );

    state.set_fee_during_setup_phase(SECOND_TEST_TOKEN, WantedFeeType::Fixed, Some(INVALID_FEE));

    state.set_fee_during_setup_phase(
        FIRST_TEST_TOKEN,
        WantedFeeType::AnyTokenWrong,
        Some(INVALID_TOKEN_ID),
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
    let mut state = FeeMarketTestState::new();

    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::FeeMarket, ScArray::ChainConfig]);

    let fee = FeeStruct {
        base_token: FIRST_TEST_TOKEN.to_token_identifier(),
        fee_type: FeeType::None,
    };

    state.set_fee(
        &ManagedBuffer::new(),
        &fee,
        Some(SETUP_PHASE_NOT_COMPLETED),
        None,
    );
}

/// ### TEST
/// F-MARKET_SET_FEE_FAIL
///
/// ### ACTION
/// Call `set_fee()` when operation is not registered
///
/// ### EXPECTED
/// Error CURRENT_OPERATION_NOT_REGISTERED
#[test]
fn test_set_fee_invalid_fee_type() {
    let mut state = FeeMarketTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::FeeMarket, ScArray::ChainConfig]);

    state.common_setup.complete_fee_market_setup_phase(None);

    let fee = FeeStruct {
        base_token: FIRST_TEST_TOKEN.to_token_identifier(),
        fee_type: FeeType::None,
    };

    let fee_hash = fee.generate_hash();

    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&fee_hash.to_vec()));

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let bitmap = ManagedBuffer::new();
    let epoch = 0;

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        ManagedBuffer::new(),
        &hash_of_hashes,
        bitmap,
        epoch,
        MultiValueEncoded::from_iter(vec![fee_hash]),
    );

    state.set_fee(&hash_of_hashes, &fee, Some(INVALID_FEE_TYPE), None);
}

/// ### TEST
/// F-MARKET_SET_FEE_FAIL
///
/// ### ACTION
/// Call `set_fee()` when operation is not registered
///
/// ### EXPECTED
/// Error CURRENT_OPERATION_NOT_REGISTERED
#[test]
fn test_set_fee_operation_not_registered() {
    let mut state = FeeMarketTestState::new();

    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::FeeMarket]);

    state.common_setup.complete_fee_market_setup_phase(None);

    let fee = FeeStruct {
        base_token: FIRST_TEST_TOKEN.to_token_identifier(),
        fee_type: FeeType::None,
    };

    state.set_fee(
        &ManagedBuffer::new(),
        &fee,
        Some(CURRENT_OPERATION_NOT_REGISTERED),
        None,
    );
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
    let mut state = FeeMarketTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::FeeMarket, ScArray::ChainConfig]);

    state.common_setup.complete_fee_market_setup_phase(None);

    let fee = FeeStruct {
        base_token: FIRST_TEST_TOKEN.to_token_identifier(),
        fee_type: FeeType::Fixed {
            token: FIRST_TEST_TOKEN.to_token_identifier(),
            per_transfer: BigUint::default(),
            per_gas: BigUint::default(),
        },
    };

    let fee_hash = fee.generate_hash();

    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&fee_hash.to_vec()));

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let bitmap = ManagedBuffer::new();
    let epoch = 0;

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        ManagedBuffer::new(),
        &hash_of_hashes,
        bitmap,
        epoch,
        MultiValueEncoded::from_iter(vec![fee_hash]),
    );

    state.set_fee(&hash_of_hashes, &fee, None, Some("executedBridgeOp"));

    state
        .common_setup
        .world
        .query()
        .to(FEE_MARKET_ADDRESS)
        .whitebox(fee_market::contract_obj, |sc| {
            assert!(!sc
                .token_fee(&FIRST_TEST_TOKEN.to_token_identifier())
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
    let mut state = FeeMarketTestState::new();

    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::FeeMarket]);

    state.remove_fee(
        &ManagedBuffer::new(),
        FIRST_TEST_TOKEN,
        Some(SETUP_PHASE_NOT_COMPLETED),
        None,
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
    let mut state = FeeMarketTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig, ScArray::FeeMarket]);

    state.common_setup.complete_fee_market_setup_phase(None);

    let fee = FeeStruct {
        base_token: FIRST_TEST_TOKEN.to_token_identifier(),
        fee_type: FeeType::Fixed {
            token: FIRST_TEST_TOKEN.to_token_identifier(),
            per_transfer: BigUint::default(),
            per_gas: BigUint::default(),
        },
    };

    let register_fee_hash = fee.generate_hash();

    let register_fee_hash_of_hashes =
        ManagedBuffer::new_from_bytes(&sha256(&register_fee_hash.to_vec()));

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let bitmap = ManagedBuffer::new();
    let epoch = 0;

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        ManagedBuffer::new(),
        &register_fee_hash_of_hashes,
        bitmap.clone(),
        epoch,
        MultiValueEncoded::from_iter(vec![register_fee_hash]),
    );

    state.set_fee(
        &register_fee_hash_of_hashes,
        &fee,
        None,
        Some("executedBridgeOp"),
    );

    state
        .common_setup
        .world
        .query()
        .to(FEE_MARKET_ADDRESS)
        .whitebox(fee_market::contract_obj, |sc| {
            assert!(!sc
                .token_fee(&FIRST_TEST_TOKEN.to_token_identifier())
                .is_empty());
        });

    let remove_fee_hash = sha256(
        &FIRST_TEST_TOKEN
            .to_token_identifier::<StaticApi>()
            .as_managed_buffer()
            .to_vec(),
    );

    let remove_fee_hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&remove_fee_hash));

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        ManagedBuffer::new(),
        &remove_fee_hash_of_hashes,
        bitmap,
        epoch,
        MultiValueEncoded::from_iter(vec![ManagedBuffer::new_from_bytes(&remove_fee_hash)]),
    );

    state.remove_fee(
        &remove_fee_hash_of_hashes,
        FIRST_TEST_TOKEN,
        None,
        Some("executedBridgeOp"),
    );

    state
        .common_setup
        .world
        .query()
        .to(FEE_MARKET_ADDRESS)
        .whitebox(fee_market::contract_obj, |sc| {
            assert!(sc
                .token_fee(&FIRST_TEST_TOKEN.to_token_identifier())
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
    let mut state = FeeMarketTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig, ScArray::FeeMarket]);

    state.common_setup.complete_fee_market_setup_phase(None);

    let fee = FeeStruct {
        base_token: FIRST_TEST_TOKEN.to_token_identifier(),
        fee_type: FeeType::Fixed {
            token: FIRST_TEST_TOKEN.to_token_identifier(),
            per_transfer: BigUint::default(),
            per_gas: BigUint::default(),
        },
    };

    let remove_fee_hash: ManagedBuffer<StaticApi> = ManagedBuffer::new_from_bytes(&sha256(
        &FIRST_TEST_TOKEN
            .to_token_identifier::<StaticApi>()
            .as_managed_buffer()
            .to_vec(),
    ));
    let register_fee_hash = fee.generate_hash();
    let mut aggregated_hashes = ManagedBuffer::new();

    aggregated_hashes.append(&remove_fee_hash);
    aggregated_hashes.append(&register_fee_hash);

    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&aggregated_hashes.to_vec()));

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let bitmap = ManagedBuffer::new();
    let epoch = 0;

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        ManagedBuffer::new(),
        &hash_of_hashes,
        bitmap,
        epoch,
        MultiValueEncoded::from_iter(vec![remove_fee_hash, register_fee_hash]),
    );

    state.set_fee(&hash_of_hashes, &fee, None, Some("executedBridgeOp"));

    state
        .common_setup
        .world
        .query()
        .to(FEE_MARKET_ADDRESS)
        .whitebox(fee_market::contract_obj, |sc| {
            assert!(!sc
                .token_fee(&FIRST_TEST_TOKEN.to_token_identifier())
                .is_empty());
        });

    state.remove_fee(
        &hash_of_hashes,
        FIRST_TEST_TOKEN,
        None,
        Some("executedBridgeOp"),
    );

    state
        .common_setup
        .world
        .query()
        .to(FEE_MARKET_ADDRESS)
        .whitebox(fee_market::contract_obj, |sc| {
            assert!(sc
                .token_fee(&FIRST_TEST_TOKEN.to_token_identifier())
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
    let mut state = FeeMarketTestState::new();

    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::FeeMarket]);

    state.distribute_fees(&ManagedBuffer::new(), vec![], Some(CALLER_NOT_OWNER), None);
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
    let mut state = FeeMarketTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);

    state.common_setup.complete_fee_market_setup_phase(None);

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig, ScArray::FeeMarket]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    state.distribute_fees(
        &ManagedBuffer::new(),
        vec![],
        Some(CURRENT_OPERATION_NOT_REGISTERED),
        None,
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
fn distribute_fees_percentage_under_limit() {
    let mut state = FeeMarketTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);

    state.common_setup.complete_fee_market_setup_phase(None);

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig, ScArray::FeeMarket]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let address_pair: AddressPercentagePair<StaticApi> = AddressPercentagePair {
        address: OWNER_ADDRESS.to_managed_address(),
        percentage: 10,
    };

    let address_pair_tuple =
        MultiValue2::from((address_pair.address.clone(), address_pair.percentage));
    let address_pair_hash = address_pair.generate_hash();
    let pair_hash_byte_array = ManagedBuffer::new_from_bytes(&sha256(&address_pair_hash.to_vec()));
    let mut aggregated_hash: ManagedBuffer<StaticApi> = ManagedBuffer::new();
    aggregated_hash.append(&pair_hash_byte_array);

    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&aggregated_hash.to_vec()));

    let bitmap = ManagedBuffer::new();
    let epoch = 0;

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        ManagedBuffer::new(),
        &hash_of_hashes,
        bitmap,
        epoch,
        MultiValueEncoded::from_iter(vec![pair_hash_byte_array]),
    );

    state.distribute_fees(
        &hash_of_hashes,
        vec![address_pair_tuple],
        None,
        Some("failedBridgeOp"),
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
fn distribute_fees() {
    let mut state = FeeMarketTestState::new();

    state
        .common_setup
        .deploy_chain_config(OptionalValue::None, None);

    let fee_per_transfer = BigUint::from(100u32);

    let fee = FeeStruct {
        base_token: FIRST_TEST_TOKEN.to_token_identifier(),
        fee_type: FeeType::Fixed {
            token: FIRST_TEST_TOKEN.to_token_identifier(),
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

    state.common_setup.complete_fee_market_setup_phase(None);

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ChainConfig, ScArray::FeeMarket]);

    state
        .common_setup
        .complete_header_verifier_setup_phase(None);

    let address_pair: AddressPercentagePair<StaticApi> = AddressPercentagePair {
        address: OWNER_ADDRESS.to_managed_address(),
        percentage: 10_000,
    };

    let address_pair_tuple =
        MultiValue2::from((address_pair.address.clone(), address_pair.percentage));
    let address_pair_hash = address_pair.generate_hash();
    let pair_hash_byte_array = ManagedBuffer::new_from_bytes(&sha256(&address_pair_hash.to_vec()));
    let mut aggregated_hash: ManagedBuffer<StaticApi> = ManagedBuffer::new();
    aggregated_hash.append(&pair_hash_byte_array);

    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&aggregated_hash.to_vec()));
    let bitmap = ManagedBuffer::new();
    let epoch = 0;

    state.common_setup.register_operation(
        OWNER_ADDRESS,
        ManagedBuffer::new(),
        &hash_of_hashes,
        bitmap,
        epoch,
        MultiValueEncoded::from_iter(vec![pair_hash_byte_array]),
    );

    state.distribute_fees(
        &hash_of_hashes,
        vec![address_pair_tuple],
        None,
        Some("executedBridgeOp"),
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
    let mut state = FeeMarketTestState::new();

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
    let mut state = FeeMarketTestState::new();

    let fee = state.get_fee();

    state
        .common_setup
        .deploy_fee_market(Some(fee), ESDT_SAFE_ADDRESS);

    let whitelisted_users = vec![USER_ADDRESS];

    state.add_users_to_whitelist(whitelisted_users);

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
    let mut state = FeeMarketTestState::new();

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
    let mut state = FeeMarketTestState::new();

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
    let mut state = FeeMarketTestState::new();

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
