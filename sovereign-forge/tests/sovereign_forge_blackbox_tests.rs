use chain_config::storage::ChainConfigStorageModule;
use common_test_setup::{
    base_setup::helpers::BLSKey,
    constants::{
        CHAIN_FACTORY_SC_ADDRESS, CHAIN_ID, DEPLOY_COST, ESDT_SAFE_ADDRESS, FIRST_TEST_TOKEN,
        NATIVE_TEST_TOKEN, ONE_HUNDRED_THOUSAND, OWNER_ADDRESS, SOVEREIGN_FORGE_SC_ADDRESS,
        USER_ADDRESS,
    },
};
use cross_chain::storage::CrossChainStorage;
use error_messages::{
    CALLER_DID_NOT_DEPLOY_ANY_SOV_CHAIN, CHAIN_CONFIG_ALREADY_DEPLOYED, CHAIN_ID_ALREADY_IN_USE,
    CHAIN_ID_NOT_LOWERCASE_ALPHANUMERIC, DEPLOY_COST_NOT_ENOUGH, ESDT_SAFE_ALREADY_DEPLOYED,
    ESDT_SAFE_NOT_DEPLOYED, FEE_MARKET_ALREADY_DEPLOYED, FEE_MARKET_NOT_DEPLOYED,
    HEADER_VERIFIER_ALREADY_DEPLOYED, HEADER_VERIFIER_NOT_DEPLOYED, INVALID_CHAIN_ID,
};
use fee_common::storage::FeeCommonStorageModule;
use multiversx_sc::{
    imports::OptionalValue,
    types::{
        BigUint, EgldOrEsdtTokenIdentifier, ManagedBuffer, ManagedVec, MultiEgldOrEsdtPayment,
        ReturnsResultUnmanaged,
    },
};
use multiversx_sc_scenario::{ScenarioTxRun, ScenarioTxWhitebox};
use proxies::chain_config_proxy::ChainConfigContractProxy;
use sovereign_forge::forge_common::{forge_utils::ForgeUtilsModule, storage::StorageModule};
use sovereign_forge_blackbox_setup::SovereignForgeTestState;
use structs::{
    configs::{EsdtSafeConfig, SovereignConfig},
    fee::{FeeStruct, FeeType},
    forge::ScArray,
};
mod sovereign_forge_blackbox_setup;

/// ### TEST
/// S-FORGE_DEPLOY_OK
///
/// ### ACTION
/// Deploy sovereign_forge and chain_factory
///
/// ### EXPECTED
/// Both sovereign_forge and chain_factory contracts deploy successfully
#[test]
fn test_deploy_contracts() {
    let mut state = SovereignForgeTestState::new();
    state
        .common_setup
        .deploy_sovereign_forge(OptionalValue::None);
    state.common_setup.deploy_chain_factory();
}

/// ### TEST
/// S-FORGE_REGISTER_CHAIN_FACTORY_OK
///
/// ### ACTION
/// Register chain_factory any shard
///
/// ### EXPECTED
/// chain_factories() storage non-empty
#[test]
fn test_register_chain_factory() {
    let mut state = SovereignForgeTestState::new();
    state
        .common_setup
        .deploy_sovereign_forge(OptionalValue::None);

    state.register_chain_factory(2, CHAIN_FACTORY_SC_ADDRESS);

    state
        .common_setup
        .world
        .query()
        .to(SOVEREIGN_FORGE_SC_ADDRESS)
        .whitebox(sovereign_forge::contract_obj, |sc| {
            assert!(!sc.chain_factories(2).is_empty());
        });
}

/// ### TEST
/// S-FORGE_UPDATE_CONFIG_FAIL
///
/// ### ACTION
/// Update config without deploying chain_config
///
/// ### EXPECTED
/// Error CALLER_DID_NOT_DEPLOY_ANY_SOV_CHAIN
#[test]
fn test_update_sovereign_config_no_chain_config_deployed() {
    let mut state = SovereignForgeTestState::new();
    state
        .common_setup
        .deploy_sovereign_forge(OptionalValue::None);

    state.register_chain_factory(2, CHAIN_FACTORY_SC_ADDRESS);

    state
        .common_setup
        .world
        .query()
        .to(SOVEREIGN_FORGE_SC_ADDRESS)
        .whitebox(sovereign_forge::contract_obj, |sc| {
            assert!(!sc.chain_factories(2).is_empty());
        });

    state.update_sovereign_config(
        SovereignConfig::default_config(),
        Some(CALLER_DID_NOT_DEPLOY_ANY_SOV_CHAIN),
    );
}

/// ### TEST
/// S-FORGE_UPDATE_CONFIG_OK
///
/// ### ACTION
/// Update sovereign config
///
/// ### EXPECTED
/// Sovereign config was modified
#[test]
fn test_update_sovereign_config() {
    let mut state = SovereignForgeTestState::new();

    state
        .common_setup
        .deploy_sovereign_forge(OptionalValue::None);

    state.deploy_template_scs(Some(vec![
        ScArray::ChainFactory,
        ScArray::ChainConfig,
        ScArray::ESDTSafe,
        ScArray::FeeMarket,
    ]));

    state.finish_setup();

    state
        .common_setup
        .world
        .query()
        .to(SOVEREIGN_FORGE_SC_ADDRESS)
        .whitebox(sovereign_forge::contract_obj, |sc| {
            assert!(!sc.chain_factories(2).is_empty());
        });

    let deploy_cost = BigUint::from(DEPLOY_COST);

    state.common_setup.deploy_phase_one(
        &deploy_cost,
        Some(ManagedBuffer::from(CHAIN_ID)),
        OptionalValue::None,
        None,
    );

    state
        .common_setup
        .world
        .query()
        .to(SOVEREIGN_FORGE_SC_ADDRESS)
        .whitebox(sovereign_forge::contract_obj, |sc| {
            assert!(!sc
                .sovereigns_mapper(&OWNER_ADDRESS.to_managed_address())
                .is_empty());

            assert!(sc.chain_ids().contains(&ManagedBuffer::from(CHAIN_ID)));

            let is_chain_config_deployed =
                sc.is_contract_deployed(&OWNER_ADDRESS.to_managed_address(), ScArray::ChainConfig);
            assert!(is_chain_config_deployed);
        });

    state.update_sovereign_config(SovereignConfig::new(1, 2, BigUint::default(), None), None);

    let chain_config_address_from_sovereign_forge = state
        .get_smart_contract_address_from_sovereign_forge(
            ManagedBuffer::from(CHAIN_ID),
            ScArray::ChainConfig,
        );

    state
        .common_setup
        .world
        .query()
        .to(chain_config_address_from_sovereign_forge)
        .whitebox(chain_config::contract_obj, |sc| {
            let min_validators = sc.sovereign_config().get().min_validators;
            assert!(min_validators == 1);
        })
}

/// ### TEST
/// S-FORGE_UPDATE_ESDT_SAFE_CONFIG_OK
///
/// ### ACTION
/// Update ESDT safe config
///
/// ### EXPECTED
/// ESDT safe config was modified
#[test]
fn test_update_esdt_safe_config() {
    let mut state = SovereignForgeTestState::new();
    state
        .common_setup
        .deploy_sovereign_forge(OptionalValue::None);

    state.deploy_template_scs(Some(vec![
        ScArray::ChainFactory,
        ScArray::ChainConfig,
        ScArray::ESDTSafe,
        ScArray::FeeMarket,
    ]));

    state.finish_setup();

    state
        .common_setup
        .world
        .query()
        .to(SOVEREIGN_FORGE_SC_ADDRESS)
        .whitebox(sovereign_forge::contract_obj, |sc| {
            assert!(!sc.chain_factories(2).is_empty());
        });

    let deploy_cost = BigUint::from(DEPLOY_COST);

    state.common_setup.deploy_phase_one(
        &deploy_cost,
        Some(ManagedBuffer::from(CHAIN_ID)),
        OptionalValue::None,
        None,
    );
    state
        .common_setup
        .world
        .query()
        .to(SOVEREIGN_FORGE_SC_ADDRESS)
        .whitebox(sovereign_forge::contract_obj, |sc| {
            assert!(!sc
                .sovereigns_mapper(&OWNER_ADDRESS.to_managed_address())
                .is_empty());

            assert!(sc.chain_ids().contains(&ManagedBuffer::from(CHAIN_ID)));

            let is_chain_config_deployed =
                sc.is_contract_deployed(&OWNER_ADDRESS.to_managed_address(), ScArray::ChainConfig);
            assert!(is_chain_config_deployed);
        });

    state
        .common_setup
        .deploy_header_verifier(vec![ScArray::ESDTSafe]);

    state
        .common_setup
        .deploy_phase_two(None, OptionalValue::None);
    state
        .common_setup
        .world
        .query()
        .to(SOVEREIGN_FORGE_SC_ADDRESS)
        .whitebox(sovereign_forge::contract_obj, |sc| {
            let is_esdt_safe_deployed =
                sc.is_contract_deployed(&OWNER_ADDRESS.to_managed_address(), ScArray::ESDTSafe);

            assert!(is_esdt_safe_deployed);
        });

    state.update_esdt_safe_config(
        EsdtSafeConfig::new(
            ManagedVec::new(),
            ManagedVec::new(),
            ONE_HUNDRED_THOUSAND.into(),
            ManagedVec::new(),
            ManagedVec::new(),
        ),
        None,
    );

    let mvx_esdt_safe_address_from_sovereign_forge = state
        .get_smart_contract_address_from_sovereign_forge(
            ManagedBuffer::from(CHAIN_ID),
            ScArray::ESDTSafe,
        );

    state
        .common_setup
        .world
        .query()
        .to(mvx_esdt_safe_address_from_sovereign_forge)
        .whitebox(mvx_esdt_safe::contract_obj, |sc| {
            let max_bridged_amount = sc.esdt_safe_config().get().max_tx_gas_limit;
            let expected_amount: u64 = ONE_HUNDRED_THOUSAND.into();
            assert!(max_bridged_amount == expected_amount);
        })
}

/// ### TEST
/// S-FORGE_SET_FEE_OK
///
/// ### ACTION
/// Set sovereign fee
///
/// ### EXPECTED
/// The sovereign fee is modified
#[test]
fn test_set_fee() {
    let mut state = SovereignForgeTestState::new();
    state
        .common_setup
        .deploy_sovereign_forge(OptionalValue::None);

    state.deploy_template_scs(Some(vec![
        ScArray::ChainFactory,
        ScArray::ChainConfig,
        ScArray::ESDTSafe,
        ScArray::FeeMarket,
        ScArray::HeaderVerifier,
    ]));

    state.finish_setup();

    let deploy_cost = BigUint::from(100_000u32);
    state.common_setup.deploy_phase_one(
        &deploy_cost,
        Some(ManagedBuffer::from(CHAIN_ID)),
        OptionalValue::None,
        None,
    );

    state
        .common_setup
        .deploy_phase_two(None, OptionalValue::None);
    state.common_setup.deploy_phase_three(None, None);
    state.common_setup.deploy_phase_four(None);

    state
        .common_setup
        .world
        .query()
        .to(SOVEREIGN_FORGE_SC_ADDRESS)
        .whitebox(sovereign_forge::contract_obj, |sc| {
            let is_chain_config_deployed =
                sc.is_contract_deployed(&OWNER_ADDRESS.to_managed_address(), ScArray::ChainConfig);
            let is_header_verifier_deployed = sc
                .is_contract_deployed(&OWNER_ADDRESS.to_managed_address(), ScArray::HeaderVerifier);
            let is_esdt_safe_deployed =
                sc.is_contract_deployed(&OWNER_ADDRESS.to_managed_address(), ScArray::ESDTSafe);
            let is_fee_market_deployed =
                sc.is_contract_deployed(&OWNER_ADDRESS.to_managed_address(), ScArray::FeeMarket);

            assert!(
                is_chain_config_deployed
                    && is_header_verifier_deployed
                    && is_esdt_safe_deployed
                    && is_fee_market_deployed
            );
        });

    let fee_type = FeeType::Fixed {
        token: EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN),
        per_transfer: BigUint::default(),
        per_gas: BigUint::default(),
    };

    let new_fee = FeeStruct {
        base_token: EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN),
        fee_type,
    };

    state.set_fee(new_fee, None);

    let fee_market_address = state.get_smart_contract_address_from_sovereign_forge(
        ManagedBuffer::from(CHAIN_ID),
        ScArray::FeeMarket,
    );

    state
        .common_setup
        .world
        .query()
        .to(fee_market_address)
        .whitebox(mvx_fee_market::contract_obj, |sc| {
            assert!(sc.is_fee_enabled());
            assert!(!sc
                .token_fee(&EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN))
                .is_empty());
        });
}

/// ### TEST
/// S-FORGE_SET_FEE_FAIL
///
/// ### ACTION
/// Call `set_fee()` phase three not completed
///
/// ### EXPECTED
/// Error FEE_MARKET_NOT_DEPLOYED
#[test]
fn test_set_fee_phase_three_not_completed() {
    let mut state = SovereignForgeTestState::new();
    state
        .common_setup
        .deploy_sovereign_forge(OptionalValue::None);

    state.deploy_template_scs(Some(vec![
        ScArray::ChainFactory,
        ScArray::ChainConfig,
        ScArray::ESDTSafe,
        ScArray::FeeMarket,
        ScArray::HeaderVerifier,
    ]));

    state.finish_setup();

    let deploy_cost = BigUint::from(100_000u32);
    state.common_setup.deploy_phase_one(
        &deploy_cost,
        Some(ManagedBuffer::from(CHAIN_ID)),
        OptionalValue::None,
        None,
    );
    state
        .common_setup
        .deploy_phase_two(None, OptionalValue::None);

    let fee_type = FeeType::Fixed {
        token: EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN),
        per_transfer: BigUint::default(),
        per_gas: BigUint::default(),
    };

    let new_fee = FeeStruct {
        base_token: EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN),
        fee_type,
    };

    state.set_fee(new_fee, Some(FEE_MARKET_NOT_DEPLOYED));
}
/// ### TEST
/// S-FORGE_REMOVE_FEE_OK
///
/// ### ACTION
/// Remove sovereign fee
///
/// ### EXPECTED
/// The sovereign fee is removed
#[test]
fn test_remove_fee() {
    let mut state = SovereignForgeTestState::new();
    state
        .common_setup
        .deploy_sovereign_forge(OptionalValue::None);

    state.deploy_template_scs(Some(vec![
        ScArray::ChainFactory,
        ScArray::ChainConfig,
        ScArray::ESDTSafe,
        ScArray::FeeMarket,
        ScArray::HeaderVerifier,
    ]));

    state.finish_setup();

    let deploy_cost = BigUint::from(100_000u32);
    state.common_setup.deploy_phase_one(
        &deploy_cost,
        Some(ManagedBuffer::from(CHAIN_ID)),
        OptionalValue::None,
        None,
    );
    state
        .common_setup
        .deploy_phase_two(None, OptionalValue::None);

    let fee_type = FeeType::Fixed {
        token: EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN),
        per_transfer: BigUint::default(),
        per_gas: BigUint::default(),
    };

    let fee = FeeStruct {
        base_token: EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN),
        fee_type,
    };
    state.common_setup.deploy_phase_three(Some(fee), None);

    state.common_setup.deploy_phase_four(None);

    state
        .common_setup
        .world
        .query()
        .to(SOVEREIGN_FORGE_SC_ADDRESS)
        .whitebox(sovereign_forge::contract_obj, |sc| {
            let is_chain_config_deployed =
                sc.is_contract_deployed(&OWNER_ADDRESS.to_managed_address(), ScArray::ChainConfig);
            let is_header_verifier_deployed = sc
                .is_contract_deployed(&OWNER_ADDRESS.to_managed_address(), ScArray::HeaderVerifier);
            let is_esdt_safe_deployed =
                sc.is_contract_deployed(&OWNER_ADDRESS.to_managed_address(), ScArray::ESDTSafe);
            let is_fee_market_deployed =
                sc.is_contract_deployed(&OWNER_ADDRESS.to_managed_address(), ScArray::FeeMarket);

            assert!(
                is_chain_config_deployed
                    && is_header_verifier_deployed
                    && is_esdt_safe_deployed
                    && is_fee_market_deployed
            );
        });

    state.remove_fee(FIRST_TEST_TOKEN, None);

    let fee_market_address = state.get_smart_contract_address_from_sovereign_forge(
        ManagedBuffer::from(CHAIN_ID),
        ScArray::FeeMarket,
    );

    state
        .common_setup
        .world
        .query()
        .to(fee_market_address)
        .whitebox(mvx_fee_market::contract_obj, |sc| {
            assert!(!sc.is_fee_enabled());
            assert!(sc
                .token_fee(&EgldOrEsdtTokenIdentifier::esdt(FIRST_TEST_TOKEN))
                .is_empty());
        })
}

/// ### TEST
/// S-FORGE_REMOVE_FEE_FAIL
///
/// ### ACTION
/// Call `remove_fee()` when phase three not deployed
///
/// ### EXPECTED
/// Error FEE_MARKET_NOT_DEPLOYED
#[test]
fn test_remove_fee_phase_three_not_completed() {
    let mut state = SovereignForgeTestState::new();
    state
        .common_setup
        .deploy_sovereign_forge(OptionalValue::None);

    state.deploy_template_scs(Some(vec![
        ScArray::ChainFactory,
        ScArray::ChainConfig,
        ScArray::ESDTSafe,
        ScArray::FeeMarket,
        ScArray::HeaderVerifier,
    ]));

    state.finish_setup();

    let deploy_cost = BigUint::from(100_000u32);
    state.common_setup.deploy_phase_one(
        &deploy_cost,
        Some(ManagedBuffer::from(CHAIN_ID)),
        OptionalValue::None,
        None,
    );
    state
        .common_setup
        .deploy_phase_two(None, OptionalValue::None);

    state.remove_fee(FIRST_TEST_TOKEN, Some(FEE_MARKET_NOT_DEPLOYED));
}

/// ### TEST
/// S-FORGE_COMPLETE_SETUP_PHASE
///
/// ### ACTION
/// Call setup_phase()
///
/// ### EXPECTED
/// Setup phase is completed and set in storage
#[test]
fn test_complete_setup_phase() {
    let mut state = SovereignForgeTestState::new();
    state
        .common_setup
        .deploy_sovereign_forge(OptionalValue::None);

    state.deploy_template_scs(Some(vec![
        ScArray::ChainFactory,
        ScArray::ChainConfig,
        ScArray::ESDTSafe,
        ScArray::FeeMarket,
        ScArray::HeaderVerifier,
    ]));

    state.finish_setup();

    let deploy_cost = BigUint::from(DEPLOY_COST);
    let preferred_chain_id = ManagedBuffer::from(CHAIN_ID);
    state.common_setup.deploy_phase_one(
        &deploy_cost,
        Some(preferred_chain_id.clone()),
        OptionalValue::None,
        None,
    );

    state
        .common_setup
        .deploy_phase_two(None, OptionalValue::None);
    state.common_setup.deploy_phase_three(None, None);
    state.common_setup.deploy_phase_four(None);

    state
        .common_setup
        .world
        .query()
        .to(SOVEREIGN_FORGE_SC_ADDRESS)
        .whitebox(sovereign_forge::contract_obj, |sc| {
            let is_chain_config_deployed =
                sc.is_contract_deployed(&OWNER_ADDRESS.to_managed_address(), ScArray::ChainConfig);
            let is_header_verifier_deployed = sc
                .is_contract_deployed(&OWNER_ADDRESS.to_managed_address(), ScArray::HeaderVerifier);
            let is_esdt_safe_deployed =
                sc.is_contract_deployed(&OWNER_ADDRESS.to_managed_address(), ScArray::ESDTSafe);
            let is_fee_market_deployed =
                sc.is_contract_deployed(&OWNER_ADDRESS.to_managed_address(), ScArray::FeeMarket);

            assert!(
                is_chain_config_deployed
                    && is_header_verifier_deployed
                    && is_esdt_safe_deployed
                    && is_fee_market_deployed
            );
        });

    let mvx_address = state.retrieve_deployed_mvx_esdt_safe_address(preferred_chain_id.clone());

    state
        .common_setup
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(mvx_address)
        .whitebox(mvx_esdt_safe::contract_obj, |sc| {
            sc.native_token()
                .set(NATIVE_TEST_TOKEN.to_token_identifier());
        });

    let chain_config_address =
        state.retrieve_deployed_chain_config_address(preferred_chain_id.clone());

    state
        .common_setup
        .world
        .tx()
        .from(OWNER_ADDRESS)
        .to(chain_config_address)
        .typed(ChainConfigContractProxy)
        .register(BLSKey::random())
        .payment(MultiEgldOrEsdtPayment::new())
        .returns(ReturnsResultUnmanaged)
        .run();

    state.complete_setup_phase(None);
    // NOTE: This will not work until callback fixes
    // state.check_setup_phase_completed(preferred_chain_id, true);
}

/// ### TEST
/// S-FORGE_DEPLOY_PHASE_ONE_FAIL
///
/// ### ACTION
/// deploy_phase_one with insufficient cost
///
/// ### EXPECTED
/// Error DEPLOY_COST_NOT_ENOUGH
#[test]
fn test_deploy_phase_one_deploy_cost_too_low() {
    let mut state = SovereignForgeTestState::new();
    state
        .common_setup
        .deploy_sovereign_forge(OptionalValue::Some(BigUint::from(2u32)));
    state.common_setup.deploy_chain_factory();
    state.finish_setup();

    let deploy_cost = BigUint::from(1u32);

    state.common_setup.deploy_phase_one(
        &deploy_cost,
        None,
        OptionalValue::None,
        Some(DEPLOY_COST_NOT_ENOUGH),
    );
}

/// ### TEST
/// S-FORGE_DEPLOY_PHASE_ONE_FAIL
///
/// ### ACTION
/// Call deploy_phase_one twice for same chain_config
///
/// ### EXPECTED
/// Error CHAIN_CONFIG_ALREADY_DEPLOYED
#[test]
fn test_deploy_phase_one_chain_config_already_deployed() {
    let mut state = SovereignForgeTestState::new();
    state
        .common_setup
        .deploy_sovereign_forge(OptionalValue::None);

    state.deploy_template_scs(Some(vec![ScArray::ChainFactory, ScArray::ChainConfig]));

    state.finish_setup();

    let deploy_cost = BigUint::from(DEPLOY_COST);

    state
        .common_setup
        .deploy_phase_one(&deploy_cost, None, OptionalValue::None, None);

    state.common_setup.deploy_phase_one(
        &deploy_cost,
        None,
        OptionalValue::None,
        Some(CHAIN_CONFIG_ALREADY_DEPLOYED),
    );
}

/// ### TEST
/// S-FORGE_DEPLOY_PHASE_ONE_FAIL
///
/// ### ACTION
/// Call deploy_phase_one wrong chain id format
///
/// ### EXPECTED
/// Error CHAIN_ID_NOT_LOWERCASE_ALPHANUMERIC
#[test]
fn test_deploy_phase_one_preferred_chain_id_not_lowercase_alphanumeric() {
    let mut state = SovereignForgeTestState::new();
    state
        .common_setup
        .deploy_sovereign_forge(OptionalValue::None);

    state.deploy_template_scs(Some(vec![ScArray::ChainFactory, ScArray::ChainConfig]));

    state.finish_setup();

    let deploy_cost = BigUint::from(DEPLOY_COST);

    state.common_setup.deploy_phase_one(
        &deploy_cost,
        Some(ManagedBuffer::from("CHID")),
        OptionalValue::None,
        Some(CHAIN_ID_NOT_LOWERCASE_ALPHANUMERIC),
    );
}

/// ### TEST
/// S-FORGE_DEPLOY_PHASE_ONE_FAIL
///
/// ### ACTION
/// Call deploy_phase_one wrong chain id length
///
/// ### EXPECTED
/// Error CHAIN_ID_NOT_FOUR_CHAR_LONG
#[test]
fn test_deploy_phase_one_preferred_chain_id_not_correct_length() {
    let mut state = SovereignForgeTestState::new();
    state
        .common_setup
        .deploy_sovereign_forge(OptionalValue::None);

    state.deploy_template_scs(Some(vec![ScArray::ChainFactory, ScArray::ChainConfig]));

    state.finish_setup();

    let deploy_cost = BigUint::from(DEPLOY_COST);

    state.common_setup.deploy_phase_one(
        &deploy_cost,
        Some(ManagedBuffer::from("CHAINID")),
        OptionalValue::None,
        Some(INVALID_CHAIN_ID),
    );
}

/// ### TEST
/// S-FORGE_DEPLOY_PHASE_ONE_OK
///
/// ### ACTION
/// Call deploy_phase_one with no preferred chain id
///
/// ### EXPECTED
/// Chain-Config is deployed and address is set in storage
#[test]
fn test_deploy_phase_one_no_preferred_chain_id() {
    let mut state = SovereignForgeTestState::new();
    state
        .common_setup
        .deploy_sovereign_forge(OptionalValue::None);

    state.deploy_template_scs(Some(vec![ScArray::ChainFactory, ScArray::ChainConfig]));

    state.finish_setup();

    let deploy_cost = BigUint::from(DEPLOY_COST);

    state
        .common_setup
        .deploy_phase_one(&deploy_cost, None, OptionalValue::None, None);

    state
        .common_setup
        .world
        .query()
        .to(SOVEREIGN_FORGE_SC_ADDRESS)
        .whitebox(sovereign_forge::contract_obj, |sc| {
            assert!(!sc
                .sovereigns_mapper(&OWNER_ADDRESS.to_managed_address())
                .is_empty());

            let is_chain_config_deployed =
                sc.is_contract_deployed(&OWNER_ADDRESS.to_managed_address(), ScArray::ChainConfig);
            assert!(is_chain_config_deployed);
        })
}

/// ### TEST
/// S-FORGE_DEPLOY_PHASE_ONE_OK
///
/// ### ACTION
/// Call deploy_phase_one with preferred chain id
///
/// ### EXPECTED
/// Chain-Config is deployed and address is set in storage
#[test]
fn test_deploy_phase_one_preferred_chain_id() {
    let mut state = SovereignForgeTestState::new();
    state
        .common_setup
        .deploy_sovereign_forge(OptionalValue::None);

    state.deploy_template_scs(Some(vec![ScArray::ChainFactory, ScArray::ChainConfig]));

    state.finish_setup();

    let deploy_cost = BigUint::from(DEPLOY_COST);

    state.common_setup.deploy_phase_one(
        &deploy_cost,
        Some(ManagedBuffer::from(CHAIN_ID)),
        OptionalValue::None,
        None,
    );

    state
        .common_setup
        .world
        .query()
        .to(SOVEREIGN_FORGE_SC_ADDRESS)
        .whitebox(sovereign_forge::contract_obj, |sc| {
            assert!(!sc
                .sovereigns_mapper(&OWNER_ADDRESS.to_managed_address())
                .is_empty());

            assert!(sc.chain_ids().contains(&ManagedBuffer::from(CHAIN_ID)));

            let is_chain_config_deployed =
                sc.is_contract_deployed(&OWNER_ADDRESS.to_managed_address(), ScArray::ChainConfig);
            assert!(is_chain_config_deployed);
        })
}

/// ### TEST
/// S-FORGE_DEPLOY_PHASE_ONE_FAIL
///
/// ### ACTION
/// Call deploy_phase_one with an used chain id
///
/// ### EXPECTED
/// Error CHAIN_ID_ALREADY_IN_USE
#[test]
fn test_deploy_phase_one_with_chain_id_used() {
    let mut state = SovereignForgeTestState::new();
    state
        .common_setup
        .deploy_sovereign_forge(OptionalValue::None);

    state.deploy_template_scs(Some(vec![ScArray::ChainFactory, ScArray::ChainConfig]));

    state.finish_setup();

    let deploy_cost = BigUint::from(DEPLOY_COST);

    state.common_setup.deploy_phase_one(
        &deploy_cost,
        Some(ManagedBuffer::from(CHAIN_ID)),
        OptionalValue::None,
        None,
    );

    state.common_setup.deploy_phase_one(
        &deploy_cost,
        Some(ManagedBuffer::from(CHAIN_ID)),
        OptionalValue::None,
        Some(CHAIN_ID_ALREADY_IN_USE),
    );
}

/// ### TEST
/// S-FORGE_DEPLOY_PHASE_TWO_FAIL
///
/// ### ACTION
/// Call deploy_phase_two without the first phase
///
/// ### EXPECTED
/// Error CALLER_DID_NOT_DEPLOY_ANY_SOV_CHAIN
#[test]
fn test_deploy_phase_two_without_first_phase() {
    let mut state = SovereignForgeTestState::new();
    state
        .common_setup
        .deploy_sovereign_forge(OptionalValue::None);
    state.common_setup.deploy_chain_factory();
    state.finish_setup();

    state.common_setup.deploy_phase_two(
        Some(CALLER_DID_NOT_DEPLOY_ANY_SOV_CHAIN),
        OptionalValue::None,
    );
}

/// ### TEST
/// S-FORGE_DEPLOY_PHASE_TWO_OK
///
/// ### ACTION
/// Call deploy_phase_two
///
/// ### EXPECTED
/// ESDT-Safe is deployed and address is set in the storage
#[test]
fn test_deploy_phase_two() {
    let mut state = SovereignForgeTestState::new();
    state
        .common_setup
        .deploy_sovereign_forge(OptionalValue::None);

    state.deploy_template_scs(Some(vec![
        ScArray::ChainFactory,
        ScArray::ChainConfig,
        ScArray::ESDTSafe,
    ]));
    state.finish_setup();

    let deploy_cost = BigUint::from(DEPLOY_COST);

    state
        .common_setup
        .deploy_phase_one(&deploy_cost, None, OptionalValue::None, None);

    state
        .common_setup
        .deploy_phase_two(None, OptionalValue::None);

    let mut esdt_safe_address_buffer_from_forge = [0u8; 32];

    state
        .common_setup
        .world
        .query()
        .to(SOVEREIGN_FORGE_SC_ADDRESS)
        .whitebox(sovereign_forge::contract_obj, |sc| {
            let is_esdt_safe_deployed =
                sc.is_contract_deployed(&OWNER_ADDRESS.to_managed_address(), ScArray::ESDTSafe);

            assert!(is_esdt_safe_deployed);

            esdt_safe_address_buffer_from_forge = sc
                .get_contract_address(&OWNER_ADDRESS.to_managed_address(), ScArray::ESDTSafe)
                .to_byte_array();
        });

    // NOTE: This will fail since callback inside blackbox don't work
    // state
    //     .common_setup
    //     .world
    //     .query()
    //     .to(ManagedAddress::new_from_bytes(
    //         &esdt_safe_address_buffer_from_forge,
    //     ))
    //     .whitebox(mvx_esdt_safe::contract_obj, |sc| {
    //         assert!(!sc.state.common_setup.get_native_token()().is_empty());
    //     });
}

/// ### TEST
/// S-FORGE_DEPLOY_PHASE_TWO_FAIL
///
/// ### ACTION
/// Call deploy_phase_two two times
///
/// ### EXPECTED
/// Error ESDT_SAFE_ALREADY_DEPLOYED
#[test]
fn test_deploy_phase_two_esdt_safe_already_deployed() {
    let mut state = SovereignForgeTestState::new();
    state
        .common_setup
        .deploy_sovereign_forge(OptionalValue::None);

    state.deploy_template_scs(Some(vec![
        ScArray::ChainFactory,
        ScArray::ChainConfig,
        ScArray::ESDTSafe,
    ]));

    state.finish_setup();

    let deploy_cost = BigUint::from(DEPLOY_COST);

    state
        .common_setup
        .deploy_phase_one(&deploy_cost, None, OptionalValue::None, None);

    state
        .common_setup
        .deploy_phase_two(None, OptionalValue::None);

    state
        .common_setup
        .deploy_phase_two(Some(ESDT_SAFE_ALREADY_DEPLOYED), OptionalValue::None);
}

/// ### TEST
/// S-FORGE_DEPLOY_PHASE_THREE_OK
///
/// ### ACTION
/// Call deploy_phase_three
///
/// ### EXPECTED
/// Fee-Market is deployed and address is set in storage
#[test]
fn test_deploy_phase_three() {
    let mut state = SovereignForgeTestState::new();
    state
        .common_setup
        .deploy_sovereign_forge(OptionalValue::None);

    state.deploy_template_scs(Some(vec![
        ScArray::ChainFactory,
        ScArray::ChainConfig,
        ScArray::ESDTSafe,
        ScArray::FeeMarket,
    ]));

    state.finish_setup();

    let deploy_cost = BigUint::from(DEPLOY_COST);

    state
        .common_setup
        .deploy_phase_one(&deploy_cost, None, OptionalValue::None, None);

    state
        .common_setup
        .deploy_phase_two(None, OptionalValue::None);

    state.common_setup.deploy_phase_three(None, None);

    state
        .common_setup
        .world
        .query()
        .to(SOVEREIGN_FORGE_SC_ADDRESS)
        .whitebox(sovereign_forge::contract_obj, |sc| {
            let is_fee_market_deployed =
                sc.is_contract_deployed(&OWNER_ADDRESS.to_managed_address(), ScArray::ESDTSafe);

            assert!(is_fee_market_deployed);
        })
}

/// ### TEST
/// S-FORGE_REMOVE_USERS_FROM_FEE_WHITELIST
///
/// ### ACTION
/// Call remove_users_from_whitelist
///
/// ### EXPECTED
/// Users are removed from fee-market's storage
#[test]
fn test_remove_users_from_whitelist() {
    let mut state = SovereignForgeTestState::new();
    state
        .common_setup
        .deploy_sovereign_forge(OptionalValue::None);

    state.deploy_template_scs(Some(vec![
        ScArray::ChainFactory,
        ScArray::ChainConfig,
        ScArray::ESDTSafe,
        ScArray::FeeMarket,
    ]));

    state.finish_setup();

    let deploy_cost = BigUint::from(DEPLOY_COST);

    state
        .common_setup
        .deploy_phase_one(&deploy_cost, None, OptionalValue::None, None);

    state
        .common_setup
        .deploy_phase_two(None, OptionalValue::None);

    state.common_setup.deploy_phase_three(None, None);

    state
        .common_setup
        .world
        .query()
        .to(SOVEREIGN_FORGE_SC_ADDRESS)
        .whitebox(sovereign_forge::contract_obj, |sc| {
            let is_fee_market_deployed =
                sc.is_contract_deployed(&OWNER_ADDRESS.to_managed_address(), ScArray::ESDTSafe);

            assert!(is_fee_market_deployed);
        });

    let whitelisted_users = vec![
        USER_ADDRESS.to_managed_address(),
        OWNER_ADDRESS.to_managed_address(),
    ];

    state.add_users_to_whitelist(whitelisted_users.clone());
    state
        .common_setup
        .query_user_fee_whitelist(Some(&whitelisted_users));

    let users_to_remove = vec![USER_ADDRESS.to_managed_address()];
    let expected_users = vec![OWNER_ADDRESS.to_managed_address()];

    state.remove_users_from_whitelist(users_to_remove.clone());
    state
        .common_setup
        .query_user_fee_whitelist(Some(&expected_users));
}

/// ### TEST
/// S-FORGE_DEPLOY_PHASE_THREE_FAIL
///
/// ### ACTION
/// Call deploy_phase_three without the phase one
///
/// ### EXPECTED
/// Error ESDT_SAFE_NOT_DEPLOYED
#[test]
fn test_deploy_phase_three_without_phase_one() {
    let mut state = SovereignForgeTestState::new();
    state
        .common_setup
        .deploy_sovereign_forge(OptionalValue::None);
    state.deploy_template_scs(Some(vec![ScArray::ChainFactory, ScArray::ChainConfig]));
    state.finish_setup();

    state
        .common_setup
        .deploy_phase_three(None, Some(ESDT_SAFE_NOT_DEPLOYED));
}

/// ### TEST
/// S-FORGE_DEPLOY_PHASE_THREE_FAIL
///
/// ### ACTION
/// Call deploy_phase_three without the phase two
///
/// ### EXPECTED
/// Error ESDT_SAFE_NOT_DEPLOYED
#[test]
fn test_deploy_phase_three_without_phase_two() {
    let mut state = SovereignForgeTestState::new();
    state
        .common_setup
        .deploy_sovereign_forge(OptionalValue::None);

    state.deploy_template_scs(Some(vec![
        ScArray::ChainFactory,
        ScArray::ChainConfig,
        ScArray::ESDTSafe,
        ScArray::FeeMarket,
        ScArray::HeaderVerifier,
    ]));

    state.finish_setup();

    let deploy_cost = BigUint::from(DEPLOY_COST);
    state
        .common_setup
        .deploy_phase_one(&deploy_cost, None, OptionalValue::None, None);

    state
        .common_setup
        .deploy_phase_three(None, Some(ESDT_SAFE_NOT_DEPLOYED));
}

/// ### TEST
/// S-FORGE_DEPLOY_PHASE_THREE_FAIL
///
/// ### ACTION
/// Call deploy_phase_three two times
///
/// ### EXPECTED
/// Error FEE_MARKET_ALREADY_DEPLOYED
#[test]
fn test_deploy_phase_three_already_deployed() {
    let mut state = SovereignForgeTestState::new();
    state
        .common_setup
        .deploy_sovereign_forge(OptionalValue::None);

    state.deploy_template_scs(Some(vec![
        ScArray::ChainFactory,
        ScArray::ChainConfig,
        ScArray::ESDTSafe,
        ScArray::FeeMarket,
    ]));

    state.finish_setup();

    let deploy_cost = BigUint::from(DEPLOY_COST);
    state
        .common_setup
        .deploy_phase_one(&deploy_cost, None, OptionalValue::None, None);

    state
        .common_setup
        .deploy_phase_two(None, OptionalValue::None);
    state.common_setup.deploy_phase_three(None, None);
    state
        .common_setup
        .deploy_phase_three(None, Some(FEE_MARKET_ALREADY_DEPLOYED));
}

/// ### TEST
/// S-FORGE_COMPLETE_SETUP_PHASE_FAIL
///
/// ### ACTION
/// Call complete_setup_phase without phase four deployed
///
/// ### EXPECTED
/// Error HEADER_VERIFIER_NOT_DEPLOYED
#[test]
fn test_complete_setup_phase_four_not_deployed() {
    let mut state = SovereignForgeTestState::new();
    state
        .common_setup
        .deploy_sovereign_forge(OptionalValue::None);
    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);
    state.complete_setup_phase(Some(HEADER_VERIFIER_NOT_DEPLOYED));
}

/// ### TEST
/// S-FORGE_DEPLOY_PHASE_FOUR_OK
///
/// ### ACTION
/// Call deploy_phase_four
///
/// ### EXPECTED
/// Header-Verifier is deployed and address is set in storage
#[test]
fn test_deploy_phase_four() {
    let mut state = SovereignForgeTestState::new();
    state
        .common_setup
        .deploy_sovereign_forge(OptionalValue::None);

    state.deploy_template_scs(Some(vec![
        ScArray::ChainFactory,
        ScArray::ChainConfig,
        ScArray::ESDTSafe,
        ScArray::FeeMarket,
        ScArray::HeaderVerifier,
    ]));

    state.finish_setup();

    let deploy_cost = BigUint::from(DEPLOY_COST);
    state
        .common_setup
        .deploy_phase_one(&deploy_cost, None, OptionalValue::None, None);

    state
        .common_setup
        .deploy_phase_two(None, OptionalValue::None);

    state.common_setup.deploy_phase_three(None, None);

    state.common_setup.deploy_phase_four(None);

    state
        .common_setup
        .world
        .query()
        .to(SOVEREIGN_FORGE_SC_ADDRESS)
        .whitebox(sovereign_forge::contract_obj, |sc| {
            let is_header_verifier_deployed = sc
                .is_contract_deployed(&OWNER_ADDRESS.to_managed_address(), ScArray::HeaderVerifier);

            assert!(is_header_verifier_deployed);
        })
}

/// ### TEST
/// S-FORGE_DEPLOY_PHASE_FOUR_FAIL
///
/// ### ACTION
/// Call deploy_phase_four without phase three
///
/// ### EXPECTED
/// Error FEE_MARKET_NOT_DEPLOYED
#[test]
fn test_deploy_phase_four_without_previous_phase() {
    let mut state = SovereignForgeTestState::new();
    state
        .common_setup
        .deploy_sovereign_forge(OptionalValue::None);

    state.deploy_template_scs(Some(vec![
        ScArray::ChainFactory,
        ScArray::ChainConfig,
        ScArray::ESDTSafe,
        ScArray::FeeMarket,
        ScArray::HeaderVerifier,
    ]));

    state.finish_setup();

    let deploy_cost = BigUint::from(DEPLOY_COST);

    state
        .common_setup
        .deploy_phase_one(&deploy_cost, None, OptionalValue::None, None);
    state
        .common_setup
        .deploy_phase_two(None, OptionalValue::None);
    state
        .common_setup
        .deploy_phase_four(Some(FEE_MARKET_NOT_DEPLOYED));
}

/// ### TEST
/// S-FORGE_DEPLOY_PHASE_FOUR_FAIL
///
/// ### ACTION
/// Call deploy_phase_four times
///
/// ### EXPECTED
/// Error HEADER_VERIFIER_ALREADY_DEPLOYED
#[test]
fn test_deploy_phase_four_header_verifier_already_deployed() {
    let mut state = SovereignForgeTestState::new();
    state
        .common_setup
        .deploy_sovereign_forge(OptionalValue::None);

    state.deploy_template_scs(Some(vec![
        ScArray::ChainFactory,
        ScArray::ChainConfig,
        ScArray::ESDTSafe,
        ScArray::FeeMarket,
        ScArray::HeaderVerifier,
    ]));

    state.finish_setup();

    let deploy_cost = BigUint::from(DEPLOY_COST);
    state
        .common_setup
        .deploy_phase_one(&deploy_cost, None, OptionalValue::None, None);
    state
        .common_setup
        .deploy_phase_two(None, OptionalValue::None);
    state.common_setup.deploy_phase_three(None, None);
    state.common_setup.deploy_phase_four(None);
    state
        .common_setup
        .deploy_phase_four(Some(HEADER_VERIFIER_ALREADY_DEPLOYED));
}
