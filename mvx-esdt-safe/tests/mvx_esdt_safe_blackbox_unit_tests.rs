use cross_chain::{DEFAULT_ISSUE_COST, MAX_GAS_PER_TRANSACTION};
use error_messages::{INVALID_TYPE, MAX_GAS_LIMIT_PER_TX_EXCEEDED};
use multiversx_sc::{
    imports::OptionalValue,
    types::{
        BigUint, EsdtTokenType, ManagedBuffer, ManagedVec, TestTokenIdentifier, TokenIdentifier,
    },
};
use multiversx_sc_scenario::ScenarioTxWhitebox;
use mvx_esdt_safe::briding_mechanism::{BridgingMechanism, TRUSTED_TOKEN_IDS};
use mvx_esdt_safe_blackbox_setup::{
    MvxEsdtSafeTestState, RegisterTokenArgs, ESDT_SAFE_ADDRESS, HEADER_VERIFIER_ADDRESS,
    OWNER_ADDRESS, TEST_TOKEN_ONE, TEST_TOKEN_TWO,
};
use structs::configs::EsdtSafeConfig;

mod mvx_esdt_safe_blackbox_setup;

#[test]
fn deploy() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract(
        HEADER_VERIFIER_ADDRESS,
        OWNER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );
}

#[test]
fn deploy_no_config() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OWNER_ADDRESS, OptionalValue::None);
    state
        .world
        .check_account(ESDT_SAFE_ADDRESS)
        .check_storage(
            "str:crossChainConfig",
            "0x00000000000000000000000011e1a30000000000", // default EsdtSafeConfig hex encoded
        )
        .check_storage("str:only_admin_module:admins.len", "0x01")
        .check_storage("0x6f6e6c795f61646d696e5f6d6f64756c653a61646d696e732e6974656d00000001", "0x6f776e65725f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f")
        .check_storage(
            "0x6f6e6c795f61646d696e5f6d6f64756c653a61646d696e732e696e6465786f776e65725f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f",
            "0x01",
        )
        .check_storage(
            "str:headerVerifierAddress",
            "0x000000000000000005006865616465722d76657269666965725f5f5f5f5f5f5f", // HEADER_VERIFIER_ADDRESS hex encoded, required for the check_storage to work
        );
}

#[test]
fn deploy_invalid_config() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract(
        HEADER_VERIFIER_ADDRESS,
        OWNER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );

    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        MAX_GAS_PER_TRANSACTION + 1,
        ManagedVec::new(),
    );

    state.update_configuration(config, Some(MAX_GAS_LIMIT_PER_TX_EXCEEDED));
}

#[test]
fn deploy_and_update_config() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OWNER_ADDRESS, OptionalValue::None);

    state
        .world
        .check_account(ESDT_SAFE_ADDRESS)
        .check_storage(
            "str:crossChainConfig",
            "0x00000000000000000000000011e1a30000000000", // default EsdtSafeConfig hex encoded
        )
        .check_storage("str:only_admin_module:admins.len", "0x01")
        .check_storage("0x6f6e6c795f61646d696e5f6d6f64756c653a61646d696e732e6974656d00000001", "0x6f776e65725f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f")
        .check_storage(
            "0x6f6e6c795f61646d696e5f6d6f64756c653a61646d696e732e696e6465786f776e65725f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f",
            "0x01",
        )
        .check_storage(
            "str:headerVerifierAddress",
            "0x000000000000000005006865616465722d76657269666965725f5f5f5f5f5f5f", // HEADER_VERIFIER_ADDRESS hex encoded, required for the check_storage to work
        );

    let new_config = EsdtSafeConfig {
        token_whitelist: ManagedVec::from_single_item(TokenIdentifier::from(TEST_TOKEN_ONE)),
        token_blacklist: ManagedVec::from_single_item(TokenIdentifier::from(TEST_TOKEN_TWO)),
        max_tx_gas_limit: 30_000,
        banned_endpoints: ManagedVec::from_single_item(ManagedBuffer::from("endpoint")),
    };

    state.update_configuration(new_config, None);

    state
        .world
        .check_account(ESDT_SAFE_ADDRESS)
        .check_storage(
            "str:crossChainConfig",
            "0x000000010000000b544f4e452d313233343536000000010000000b5454574f2d31323334353600000000000075300000000100000008656e64706f696e74", // updated EsdtSafeConfig hex encoded
        )
        .check_storage("str:only_admin_module:admins.len", "0x01")
        .check_storage("0x6f6e6c795f61646d696e5f6d6f64756c653a61646d696e732e6974656d00000001", "0x6f776e65725f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f")
        .check_storage(
            "0x6f6e6c795f61646d696e5f6d6f64756c653a61646d696e732e696e6465786f776e65725f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f5f",
            "0x01",
        )
        .check_storage(
            "str:headerVerifierAddress",
            "0x000000000000000005006865616465722d76657269666965725f5f5f5f5f5f5f", // HEADER_VERIFIER_ADDRESS hex encoded, required for the check_storage to work
        );
}

#[test]
fn set_token_burn_mechanism_no_roles() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract(
        HEADER_VERIFIER_ADDRESS,
        OWNER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );

    state.set_token_burn_mechanism(
        "WEGLD",
        Some("This token does not have Mint and Burn roles"),
    );
}

#[test]
fn set_token_burn_mechanism_token_not_trusted() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract_with_roles();

    state.set_token_burn_mechanism(TEST_TOKEN_ONE, Some("Token is not trusted"));
}

#[test]
fn set_token_burn_mechanism() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract_with_roles();

    state.set_token_burn_mechanism(TRUSTED_TOKEN_IDS[0], None);

    state
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .whitebox(mvx_esdt_safe::contract_obj, |sc| {
            assert!(sc
                .burn_mechanism_tokens()
                .contains(&TokenIdentifier::from(TRUSTED_TOKEN_IDS[0])))
        })
}

#[test]
fn set_token_lock_mechanism() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract_with_roles();

    state.set_token_burn_mechanism(TRUSTED_TOKEN_IDS[0], None);
    state.set_token_lock_mechanism(TRUSTED_TOKEN_IDS[0], None);

    state
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .whitebox(mvx_esdt_safe::contract_obj, |sc| {
            assert!(sc.burn_mechanism_tokens().is_empty())
        })
}

#[test]
fn register_token_invalid_type() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = OptionalValue::Some(EsdtSafeConfig::default_config());
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OWNER_ADDRESS, config);

    let sov_token_id = TestTokenIdentifier::new(TEST_TOKEN_ONE);
    let token_type = EsdtTokenType::Invalid;
    let token_display_name = "TokenOne";
    let num_decimals = 3;
    let token_ticker = TEST_TOKEN_ONE;
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    let register_token_args = RegisterTokenArgs {
        sov_token_id,
        token_type,
        token_display_name,
        token_ticker,
        num_decimals,
    };

    state.register_token(register_token_args, egld_payment, Some(INVALID_TYPE));
}

#[test]
fn register_token_fungible_token() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = OptionalValue::Some(EsdtSafeConfig::default_config());
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OWNER_ADDRESS, config);

    let sov_token_id = TestTokenIdentifier::new(TEST_TOKEN_ONE);
    let token_type = EsdtTokenType::Fungible;
    let token_display_name = "TokenOne";
    let token_ticker = TEST_TOKEN_ONE;
    let num_decimals = 3;
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    let register_token_args = RegisterTokenArgs {
        sov_token_id,
        token_type,
        token_display_name,
        token_ticker,
        num_decimals,
    };

    state.register_token(register_token_args, egld_payment, None);

    // NOTE: Will use assert after framework fixes
    // state
    //     .world
    //     .query()
    //     .to(CONTRACT_ADDRESS)
    //     .whitebox(mvx_esdt_safe::contract_obj, |sc| {
    //         assert!(!sc
    //             .sovereign_to_multiversx_token_id_mapper(
    //                 &TestTokenIdentifier::new(TEST_TOKEN_ONE).into()
    //             )
    //             .is_empty());
    //     })
}

#[test]
fn register_token_nonfungible_token() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = OptionalValue::Some(EsdtSafeConfig::default_config());
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OWNER_ADDRESS, config);

    let sov_token_id = TestTokenIdentifier::new(TEST_TOKEN_ONE);
    let token_type = EsdtTokenType::NonFungible;
    let token_display_name = "TokenOne";
    let num_decimals = 0;
    let token_ticker = TEST_TOKEN_ONE;
    let egld_payment = BigUint::from(DEFAULT_ISSUE_COST);

    let register_token_args = RegisterTokenArgs {
        sov_token_id,
        token_type,
        token_display_name,
        token_ticker,
        num_decimals,
    };

    state.register_token(register_token_args, egld_payment, None);

    // NOTE: Will use assert after framework fixes
    // state
    //     .world
    //     .query()
    //     .to(CONTRACT_ADDRESS)
    //     .whitebox(mvx_esdt_safe::contract_obj, |sc| {
    //         assert!(!sc
    //             .sovereign_to_multiversx_token_id_mapper(
    //                 &TestTokenIdentifier::new(TEST_TOKEN_ONE).into()
    //             )
    //             .is_empty());
    //     })
}
