use error_messages::NO_ESDT_SAFE_ADDRESS;
use header_verifier::{Headerverifier, OperationHashStatus};
use multiversx_sc::{
    imports::OptionalValue,
    types::{
        BigUint, EsdtTokenData, ManagedBuffer, ManagedVec, MultiValueEncoded, TokenIdentifier,
    },
};
use multiversx_sc_modules::transfer_role_proxy::PaymentsVec;
use multiversx_sc_scenario::{
    api::StaticApi, multiversx_chain_vm::crypto_functions::sha256, ScenarioTxWhitebox,
};
use mvx_esdt_safe::briding_mechanism::TRUSTED_TOKEN_IDS;
use mvx_esdt_safe_blackbox_setup::{
    MvxEsdtSafeTestState, ESDT_SAFE_ADDRESS, FEE_MARKET_ADDRESS, HEADER_VERIFIER_ADDRESS,
    OWNER_ADDRESS, TESTING_SC_ADDRESS, TEST_TOKEN_ONE, USER,
};
use structs::{
    configs::{EsdtSafeConfig, SovereignConfig},
    operation::{Operation, OperationData, OperationEsdtPayment, TransferData},
};
mod mvx_esdt_safe_blackbox_setup;

#[test]
fn execute_operation_no_esdt_safe_registered() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = OptionalValue::Some(EsdtSafeConfig::default_config());
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OWNER_ADDRESS, config);

    let payment = OperationEsdtPayment::new(
        TokenIdentifier::from(TEST_TOKEN_ONE),
        0,
        EsdtTokenData::default(),
    );

    let operation_data = OperationData::new(1, OWNER_ADDRESS.to_managed_address(), None);

    let operation = Operation::new(
        TESTING_SC_ADDRESS.to_managed_address(),
        vec![payment].into(),
        operation_data,
    );

    let hash_of_hashes = state.get_operation_hash(&operation);

    state.deploy_header_verifier();

    state.execute_operation(hash_of_hashes, operation, Some(NO_ESDT_SAFE_ADDRESS));
}

#[test]
fn execute_operation_success() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = OptionalValue::Some(EsdtSafeConfig::default_config());
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OWNER_ADDRESS, config);

    let token_data = EsdtTokenData {
        amount: BigUint::from(100u64),
        ..Default::default()
    };

    let payment = OperationEsdtPayment::new(TokenIdentifier::from(TEST_TOKEN_ONE), 0, token_data);

    let gas_limit = 1;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = TransferData::new(gas_limit, function, args);

    let operation_data =
        OperationData::new(1, OWNER_ADDRESS.to_managed_address(), Some(transfer_data));

    let operation = Operation::new(
        TESTING_SC_ADDRESS.to_managed_address(),
        vec![payment].into(),
        operation_data,
    );

    let operation_hash = state.get_operation_hash(&operation);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

    state.deploy_header_verifier();
    state.deploy_testing_sc();
    state.set_esdt_safe_address_in_header_verifier(ESDT_SAFE_ADDRESS);

    let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));

    state.deploy_chain_config(SovereignConfig::default_config());
    state.register_operation(ManagedBuffer::new(), &hash_of_hashes, operations_hashes);

    state
        .world
        .query()
        .to(HEADER_VERIFIER_ADDRESS)
        .whitebox(header_verifier::contract_obj, |sc| {
            let operation_hash_whitebox = ManagedBuffer::new_from_bytes(&operation_hash.to_vec());
            let hash_of_hashes =
                ManagedBuffer::new_from_bytes(&sha256(&operation_hash_whitebox.to_vec()));

            assert!(
                sc.operation_hash_status(&hash_of_hashes, &operation_hash_whitebox)
                    .get()
                    == OperationHashStatus::NotLocked
            );
        });

    state.execute_operation(hash_of_hashes, operation.clone(), None);

    state
        .world
        .query()
        .to(HEADER_VERIFIER_ADDRESS)
        .whitebox(header_verifier::contract_obj, |sc| {
            let operation_hash_whitebox = ManagedBuffer::new_from_bytes(&operation_hash.to_vec());
            let hash_of_hashes =
                ManagedBuffer::new_from_bytes(&sha256(&operation_hash_whitebox.to_vec()));

            assert!(sc
                .operation_hash_status(&hash_of_hashes, &operation_hash_whitebox)
                .is_empty());
        })
}

#[test]
fn execute_operation_burn_mechanism_without_deposit() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract_with_roles();

    let token_data = EsdtTokenData {
        amount: BigUint::from(100u64),
        ..Default::default()
    };

    let payment =
        OperationEsdtPayment::new(TokenIdentifier::from(TRUSTED_TOKEN_IDS[0]), 0, token_data);

    let operation_data = OperationData::new(1, OWNER_ADDRESS.to_managed_address(), None);

    let operation = Operation::new(
        TESTING_SC_ADDRESS.to_managed_address(),
        vec![payment].into(),
        operation_data,
    );

    let operation_hash = state.get_operation_hash(&operation);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

    state.deploy_header_verifier();
    state.deploy_testing_sc();
    state.set_esdt_safe_address_in_header_verifier(ESDT_SAFE_ADDRESS);

    let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));

    state.deploy_chain_config(SovereignConfig::default_config());
    state.register_operation(ManagedBuffer::new(), &hash_of_hashes, operations_hashes);

    state
        .world
        .query()
        .to(HEADER_VERIFIER_ADDRESS)
        .whitebox(header_verifier::contract_obj, |sc| {
            let operation_hash_whitebox = ManagedBuffer::new_from_bytes(&operation_hash.to_vec());
            let hash_of_hashes =
                ManagedBuffer::new_from_bytes(&sha256(&operation_hash_whitebox.to_vec()));

            assert!(
                sc.operation_hash_status(&hash_of_hashes, &operation_hash_whitebox)
                    .get()
                    == OperationHashStatus::NotLocked
            );
        });

    state.set_token_burn_mechanism(TRUSTED_TOKEN_IDS[0], None);

    state.execute_operation(
        hash_of_hashes,
        operation.clone(),
        Some("cannot subtract because result would be negative"),
    );
}

#[test]
fn execute_operation_success_burn_mechanism() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract_with_roles();

    let token_data = EsdtTokenData {
        amount: BigUint::from(100u64),
        ..Default::default()
    };

    let payment =
        OperationEsdtPayment::new(TokenIdentifier::from(TRUSTED_TOKEN_IDS[0]), 0, token_data);

    let operation_data = OperationData::new(1, OWNER_ADDRESS.to_managed_address(), None);

    let operation = Operation::new(
        TESTING_SC_ADDRESS.to_managed_address(),
        vec![payment.clone()].into(),
        operation_data,
    );

    let operation_hash = state.get_operation_hash(&operation);
    let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

    state.deploy_header_verifier();
    state.deploy_testing_sc();
    state.deploy_fee_market(None);
    state.set_fee_market_address(FEE_MARKET_ADDRESS);
    state.set_esdt_safe_address_in_header_verifier(ESDT_SAFE_ADDRESS);

    let operations_hashes = MultiValueEncoded::from(ManagedVec::from(vec![operation_hash.clone()]));

    state.set_token_burn_mechanism(TRUSTED_TOKEN_IDS[0], None);

    let logs = state.deposit_with_logs(
        USER.to_managed_address(),
        OptionalValue::None,
        PaymentsVec::from(vec![payment]),
    );

    for log in logs {
        assert!(!log.topics.is_empty());
    }

    state.deploy_chain_config(SovereignConfig::default_config());
    state.register_operation(ManagedBuffer::new(), &hash_of_hashes, operations_hashes);

    state
        .world
        .query()
        .to(HEADER_VERIFIER_ADDRESS)
        .whitebox(header_verifier::contract_obj, |sc| {
            let operation_hash_whitebox = ManagedBuffer::new_from_bytes(&operation_hash.to_vec());
            let hash_of_hashes =
                ManagedBuffer::new_from_bytes(&sha256(&operation_hash_whitebox.to_vec()));

            assert!(
                sc.operation_hash_status(&hash_of_hashes, &operation_hash_whitebox)
                    .get()
                    == OperationHashStatus::NotLocked
            );
        });

    state.set_token_burn_mechanism(TRUSTED_TOKEN_IDS[0], None);

    state.execute_operation(hash_of_hashes, operation.clone(), None);
}
