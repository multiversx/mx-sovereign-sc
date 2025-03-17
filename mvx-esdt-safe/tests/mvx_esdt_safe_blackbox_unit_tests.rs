use cross_chain::{storage::CrossChainStorage, DEFAULT_ISSUE_COST, MAX_GAS_PER_TRANSACTION};
use error_messages::{
    BANNED_ENDPOINT_NAME, GAS_LIMIT_TOO_HIGH, INVALID_TYPE, MAX_GAS_LIMIT_PER_TX_EXCEEDED,
    MINT_AND_BURN_ROLES_NOT_FOUND, NO_ESDT_SAFE_ADDRESS, PAYMENT_DOES_NOT_COVER_FEE,
    TOKEN_ID_IS_NOT_TRUSTED, TOKEN_IS_FROM_SOVEREIGN, TOO_MANY_TOKENS,
};
use header_verifier::{Headerverifier, OperationHashStatus};
use multiversx_sc::contract_base::ContractBase;
use multiversx_sc::types::EgldOrEsdtTokenIdentifier;
use multiversx_sc::{
    imports::{MultiValue3, OptionalValue},
    types::{
        BigUint, EsdtTokenData, EsdtTokenPayment, EsdtTokenType, ManagedBuffer, ManagedVec,
        MultiValueEncoded, TestTokenIdentifier, TokenIdentifier,
    },
};
use multiversx_sc_scenario::{
    api::StaticApi, multiversx_chain_vm::crypto_functions::sha256, ScenarioTxWhitebox,
};
use mvx_esdt_safe::bridging_mechanism::{BridgingMechanism, TRUSTED_TOKEN_IDS};
use mvx_esdt_safe_blackbox_setup::{
    MvxEsdtSafeTestState, RegisterTokenArgs, ESDT_SAFE_ADDRESS, FEE_MARKET_ADDRESS, FEE_TOKEN,
    HEADER_VERIFIER_ADDRESS, ONE_HUNDRED_MILLION, ONE_HUNDRED_THOUSAND, OWNER_ADDRESS,
    TESTING_SC_ADDRESS, TEST_TOKEN_ONE, TEST_TOKEN_TWO, USER,
};
use proxies::fee_market_proxy::{FeeStruct, FeeType};
use structs::{
    aliases::PaymentsVec,
    configs::{EsdtSafeConfig, SovereignConfig},
    operation::{Operation, OperationData, OperationEsdtPayment, TransferData},
};

mod mvx_esdt_safe_blackbox_setup;

#[test]
fn deploy() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );
}

#[test]
fn deploy_invalid_config() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract(
        HEADER_VERIFIER_ADDRESS,
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
fn set_token_burn_mechanism_no_roles() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );

    state.set_token_burn_mechanism("WEGLD", Some(MINT_AND_BURN_ROLES_NOT_FOUND));
}

#[test]
fn set_token_burn_mechanism_token_not_trusted() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract_with_roles();

    state.set_token_burn_mechanism(TEST_TOKEN_ONE, Some(TOKEN_ID_IS_NOT_TRUSTED));
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
fn set_token_lock_mechanism_token_from_sovereign() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract_with_roles();

    state.set_token_burn_mechanism(TRUSTED_TOKEN_IDS[0], None);

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .whitebox(mvx_esdt_safe::contract_obj, |sc| {
            sc.multiversx_to_sovereign_token_id_mapper(&TokenIdentifier::from(
                TRUSTED_TOKEN_IDS[0],
            ))
            .set(TokenIdentifier::from("MOCK"));
        });

    state.set_token_lock_mechanism(TRUSTED_TOKEN_IDS[0], Some(TOKEN_IS_FROM_SOVEREIGN));
}

#[test]
fn register_token_invalid_type() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = OptionalValue::Some(EsdtSafeConfig::default_config());
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, config);

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
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, config);

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
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, config);

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

#[test]
fn deposit_nothing_to_transfer() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );
    state.deposit(
        USER.to_managed_address(),
        OptionalValue::None,
        None,
        Some("Nothing to transfer"),
    );
}

#[test]
fn deposit_too_many_tokens() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );

    let esdt_token_payment = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_ONE),
        0,
        BigUint::default(),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment; 11]);

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::None,
        Some(payments_vec),
        Some(TOO_MANY_TOKENS),
    );
}

#[test]
fn deposit_no_transfer_data() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract(
        HEADER_VERIFIER_ADDRESS,
        OptionalValue::Some(EsdtSafeConfig::default_config()),
    );
    state.deploy_fee_market(None);
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_ONE),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_TWO),
        0,
        BigUint::from(100u64),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one, esdt_token_payment_two]);

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::None,
        Some(payments_vec),
        None,
    );

    state
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .whitebox(mvx_esdt_safe::contract_obj, |sc| {
            assert!(sc
                .multiversx_to_sovereign_token_id_mapper(&TokenIdentifier::from(TEST_TOKEN_ONE))
                .is_empty());
        });
}

#[test]
fn deposit_endpoint_banned() {
    let mut state = MvxEsdtSafeTestState::new();

    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::from(vec![ManagedBuffer::from("hello")]),
    );

    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));
    state.deploy_fee_market(None);
    state.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_ONE),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_TWO),
        0,
        BigUint::from(100u64),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one, esdt_token_payment_two]);

    let gas_limit = 2;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::Some(transfer_data),
        Some(payments_vec),
        Some(BANNED_ENDPOINT_NAME),
    );
}

#[test]
fn deposit_fee_enabled() {
    let mut state = MvxEsdtSafeTestState::new();

    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::new(),
    );

    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));

    let per_transfer = BigUint::from(100u64);
    let per_gas = BigUint::from(1u64);

    let fee = FeeStruct {
        base_token: TokenIdentifier::from(FEE_TOKEN),
        fee_type: FeeType::Fixed {
            token: TokenIdentifier::from(FEE_TOKEN),
            per_transfer: per_transfer.clone(),
            per_gas: per_gas.clone(),
        },
    };

    state.deploy_fee_market(Some(fee));
    state.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let fee_amount = BigUint::from(ONE_HUNDRED_THOUSAND);

    let fee_payment =
        EsdtTokenPayment::<StaticApi>::new(TokenIdentifier::from(FEE_TOKEN), 0, fee_amount.clone());

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_ONE),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_TWO),
        0,
        BigUint::from(100u64),
    );

    let payments_vec = PaymentsVec::from(vec![
        fee_payment,
        esdt_token_payment_one.clone(),
        esdt_token_payment_two.clone(),
    ]);

    let gas_limit = 2;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::Some(transfer_data),
        Some(payments_vec.clone()),
        None,
    );

    let expected_amount_token_one =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_one.amount;

    state.world.check_account(OWNER_ADDRESS).esdt_balance(
        TokenIdentifier::from(TEST_TOKEN_ONE),
        expected_amount_token_one,
    );

    let expected_amount_token_two =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_two.amount;

    state.world.check_account(OWNER_ADDRESS).esdt_balance(
        TokenIdentifier::from(TEST_TOKEN_TWO),
        expected_amount_token_two,
    );

    let expected_amount_token_fee = BigUint::from(ONE_HUNDRED_MILLION)
        - BigUint::from(payments_vec.len() - 1) * per_transfer
        - BigUint::from(gas_limit) * per_gas;

    state
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(TokenIdentifier::from(FEE_TOKEN), expected_amount_token_fee);
}

#[test]
fn deposit_payment_doesnt_cover_fee() {
    let mut state = MvxEsdtSafeTestState::new();

    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::new(),
    );

    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));

    let fee = FeeStruct {
        base_token: TokenIdentifier::from(TEST_TOKEN_ONE),
        fee_type: FeeType::Fixed {
            token: TokenIdentifier::from(TEST_TOKEN_ONE),
            per_transfer: BigUint::from(1u64),
            per_gas: BigUint::from(1u64),
        },
    };

    state.deploy_fee_market(Some(fee));
    state.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_ONE),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_TWO),
        0,
        BigUint::from(100u64),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one, esdt_token_payment_two]);

    let gas_limit = 10_000;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::Some(transfer_data),
        Some(payments_vec),
        Some(PAYMENT_DOES_NOT_COVER_FEE),
    );
}

#[test]
fn deposit_refund() {
    let mut state = MvxEsdtSafeTestState::new();

    let config = EsdtSafeConfig::new(
        ManagedVec::new(),
        ManagedVec::new(),
        50_000_000,
        ManagedVec::new(),
    );

    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));

    let per_transfer = BigUint::from(100u64);
    let per_gas = BigUint::from(1u64);

    let fee = FeeStruct {
        base_token: TokenIdentifier::from(FEE_TOKEN),
        fee_type: FeeType::Fixed {
            token: TokenIdentifier::from(FEE_TOKEN),
            per_transfer: per_transfer.clone(),
            per_gas: per_gas.clone(),
        },
    };

    state.deploy_fee_market(Some(fee));
    state.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let fee_amount = BigUint::from(ONE_HUNDRED_THOUSAND);

    let fee_payment =
        EsdtTokenPayment::<StaticApi>::new(TokenIdentifier::from(FEE_TOKEN), 0, fee_amount.clone());

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_ONE),
        0,
        BigUint::from(ONE_HUNDRED_THOUSAND),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_TWO),
        0,
        BigUint::from(ONE_HUNDRED_THOUSAND),
    );

    let payments_vec = PaymentsVec::from(vec![
        fee_payment,
        esdt_token_payment_one.clone(),
        esdt_token_payment_two.clone(),
    ]);

    let gas_limit = 1;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    let logs = state.deposit_with_logs(
        USER.to_managed_address(),
        OptionalValue::Some(transfer_data),
        payments_vec.clone(),
    );

    for log in logs {
        assert!(!log.data.is_empty());
    }

    let expected_amount_token_one =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_one.amount;

    state.world.check_account(OWNER_ADDRESS).esdt_balance(
        TokenIdentifier::from(TEST_TOKEN_ONE),
        &expected_amount_token_one,
    );

    let expected_amount_token_two =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_two.amount;

    state.world.check_account(OWNER_ADDRESS).esdt_balance(
        TokenIdentifier::from(TEST_TOKEN_TWO),
        &expected_amount_token_two,
    );

    let expected_amount_token_fee = BigUint::from(ONE_HUNDRED_MILLION)
        - BigUint::from(payments_vec.len() - 1) * per_transfer
        - BigUint::from(gas_limit) * per_gas;

    state
        .world
        .check_account(OWNER_ADDRESS)
        .esdt_balance(TokenIdentifier::from(FEE_TOKEN), expected_amount_token_fee);
}

#[test]
fn deposit_gas_limit_too_high() {
    let mut state = MvxEsdtSafeTestState::new();

    let config = EsdtSafeConfig::new(ManagedVec::new(), ManagedVec::new(), 1, ManagedVec::new());
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, OptionalValue::Some(config));
    state.deploy_fee_market(None);
    state.deploy_testing_sc();
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    let esdt_token_payment_one = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_ONE),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_TWO),
        0,
        BigUint::from(100u64),
    );

    let payments_vec = PaymentsVec::from(vec![esdt_token_payment_one, esdt_token_payment_two]);

    let gas_limit = 2;
    let function = ManagedBuffer::<StaticApi>::from("hello");
    let args =
        ManagedVec::<StaticApi, ManagedBuffer<StaticApi>>::from(vec![ManagedBuffer::from("1")]);

    let transfer_data = MultiValue3::from((gas_limit, function, args));

    state.deposit(
        USER.to_managed_address(),
        OptionalValue::Some(transfer_data),
        Some(payments_vec),
        Some(GAS_LIMIT_TOO_HIGH),
    );
}

#[test]
fn deposit_success_burn_mechanism() {
    let mut state = MvxEsdtSafeTestState::new();

    state.deploy_contract_with_roles();
    state.deploy_fee_market(None);
    state.set_fee_market_address(FEE_MARKET_ADDRESS);

    state.set_token_burn_mechanism(TRUSTED_TOKEN_IDS[0], None);

    let esdt_token_payment_trusted_token = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TRUSTED_TOKEN_IDS[0]),
        0,
        BigUint::from(100u64),
    );

    let esdt_token_payment_two = EsdtTokenPayment::<StaticApi>::new(
        TokenIdentifier::from(TEST_TOKEN_TWO),
        0,
        BigUint::from(100u64),
    );

    let payments_vec = PaymentsVec::from(vec![
        esdt_token_payment_trusted_token.clone(),
        esdt_token_payment_two.clone(),
    ]);

    let logs =
        state.deposit_with_logs(USER.to_managed_address(), OptionalValue::None, payments_vec);

    for log in logs {
        assert!(!log.topics.is_empty());
    }

    state
        .world
        .query()
        .to(ESDT_SAFE_ADDRESS)
        .whitebox(mvx_esdt_safe::contract_obj, |sc| {
            assert!(sc
                .multiversx_to_sovereign_token_id_mapper(&TokenIdentifier::from(
                    TRUSTED_TOKEN_IDS[0]
                ))
                .is_empty());

            let expected_deposited_tokens_amount = BigUint::from(100u64);

            assert!(
                expected_deposited_tokens_amount
                    == sc
                        .deposited_tokens_amount(&TokenIdentifier::from(TRUSTED_TOKEN_IDS[0]))
                        .get()
            );

            let trusted_token_id = TokenIdentifier::from(TRUSTED_TOKEN_IDS[0]);

            assert!(!sc.deposited_tokens_amount(&trusted_token_id).is_empty());

            let trusted_token_sc_balance = sc
                .blockchain()
                .get_sc_balance(&EgldOrEsdtTokenIdentifier::esdt(trusted_token_id), 0);

            assert!(trusted_token_sc_balance == BigUint::zero());

            let second_token_sc_balance = TokenIdentifier::from(TEST_TOKEN_TWO);

            let second_token_sc_balance = sc
                .blockchain()
                .get_sc_balance(&EgldOrEsdtTokenIdentifier::esdt(second_token_sc_balance), 0);

            assert!(second_token_sc_balance == 100u64);
        });

    let expected_amount_trusted_token =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_trusted_token.amount;

    state.world.check_account(OWNER_ADDRESS).esdt_balance(
        TokenIdentifier::from(TRUSTED_TOKEN_IDS[0]),
        &expected_amount_trusted_token,
    );

    let expected_amount_token_two =
        BigUint::from(ONE_HUNDRED_MILLION) - &esdt_token_payment_two.amount;

    state.world.check_account(OWNER_ADDRESS).esdt_balance(
        TokenIdentifier::from(TEST_TOKEN_TWO),
        &expected_amount_token_two,
    );
}

#[test]
fn execute_operation_no_esdt_safe_registered() {
    let mut state = MvxEsdtSafeTestState::new();
    let config = OptionalValue::Some(EsdtSafeConfig::default_config());
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, config);

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
    state.deploy_contract(HEADER_VERIFIER_ADDRESS, config);

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
        });

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(TESTING_SC_ADDRESS)
        .whitebox(testing_sc::contract_obj, |sc| {
            let token_id = TokenIdentifier::from(TRUSTED_TOKEN_IDS[0]);

            let sc_balance = sc
                .blockchain()
                .get_sc_balance(&EgldOrEsdtTokenIdentifier::esdt(token_id), 0);

            assert!(sc_balance == BigUint::zero());
        });
}

#[test]
fn execute_operation_burn_mechanism_without_deposit_cannot_subtract() {
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

    state.set_token_burn_mechanism(TRUSTED_TOKEN_IDS[0], None);

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
        });

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .whitebox(mvx_esdt_safe::contract_obj, |sc| {
            let token_id = TokenIdentifier::from(TRUSTED_TOKEN_IDS[0]);

            assert!(sc.deposited_tokens_amount(&token_id).is_empty());

            let sc_balance = sc
                .blockchain()
                .get_sc_balance(&EgldOrEsdtTokenIdentifier::esdt(token_id), 0);

            assert!(sc_balance == BigUint::default());
        });

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(TESTING_SC_ADDRESS)
        .whitebox(testing_sc::contract_obj, |sc| {
            let token_id = TokenIdentifier::from(TRUSTED_TOKEN_IDS[0]);

            let sc_balance = sc
                .blockchain()
                .get_sc_balance(&EgldOrEsdtTokenIdentifier::esdt(token_id), 0);

            assert!(sc_balance == BigUint::zero());
        });

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(TESTING_SC_ADDRESS)
        .whitebox(testing_sc::contract_obj, |sc| {
            let token_id = TokenIdentifier::from(TRUSTED_TOKEN_IDS[0]);

            let sc_balance = sc
                .blockchain()
                .get_sc_balance(&EgldOrEsdtTokenIdentifier::esdt(token_id), 0);

            assert!(sc_balance == BigUint::zero());
        });
}

#[test]
fn execute_operation_success_burn_mechanism() {
    let mut state = MvxEsdtSafeTestState::new();
    state.deploy_contract_with_roles();

    let token_data = EsdtTokenData {
        amount: BigUint::from(100u64),
        ..Default::default()
    };

    let payment = OperationEsdtPayment::new(
        TokenIdentifier::from(TRUSTED_TOKEN_IDS[0]),
        0,
        token_data.clone(),
    );

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

    let expected_amount_trusted_token = BigUint::from(ONE_HUNDRED_MILLION) - &token_data.amount;

    state.world.check_account(OWNER_ADDRESS).esdt_balance(
        TokenIdentifier::from(TRUSTED_TOKEN_IDS[0]),
        &expected_amount_trusted_token,
    );

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(ESDT_SAFE_ADDRESS)
        .whitebox(mvx_esdt_safe::contract_obj, |sc| {
            let token_id = TokenIdentifier::from(TRUSTED_TOKEN_IDS[0]);

            assert!(sc.deposited_tokens_amount(&token_id).is_empty());

            let sc_balance = sc
                .blockchain()
                .get_sc_balance(&EgldOrEsdtTokenIdentifier::esdt(token_id), 0);

            assert!(sc_balance == 100u64);
        });

    state
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(TESTING_SC_ADDRESS)
        .whitebox(testing_sc::contract_obj, |sc| {
            let token_id = TokenIdentifier::from(TRUSTED_TOKEN_IDS[0]);

            let sc_balance = sc
                .blockchain()
                .get_sc_balance(&EgldOrEsdtTokenIdentifier::esdt(token_id), 0);

            assert!(sc_balance == BigUint::zero());
        });

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
        });
}
