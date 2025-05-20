use chain_config::validator_rules::ValidatorRulesModule;
use common_test_setup::constants::{
    CHAIN_CONFIG_ADDRESS, CHAIN_FACTORY_SC_ADDRESS, CHAIN_ID, DEPLOY_COST, ESDT_SAFE_ADDRESS,
    HEADER_VERIFIER_ADDRESS, ONE_HUNDRED_THOUSAND, OWNER_ADDRESS, SOVEREIGN_FORGE_SC_ADDRESS,
};
use cross_chain::storage::CrossChainStorage;
use error_messages::{
    CALLER_DID_NOT_DEPLOY_ANY_SOV_CHAIN, CHAIN_CONFIG_ALREADY_DEPLOYED, CHAIN_ID_ALREADY_IN_USE,
    CHAIN_ID_NOT_FOUR_CHAR_LONG, CHAIN_ID_NOT_LOWERCASE_ALPHANUMERIC, DEPLOY_COST_NOT_ENOUGH,
    ESDT_SAFE_ALREADY_DEPLOYED, ESDT_SAFE_NOT_DEPLOYED, FEE_MARKET_ALREADY_DEPLOYED,
    FEE_MARKET_NOT_DEPLOYED, HEADER_VERIFIER_ALREADY_DEPLOYED, HEADER_VERIFIER_NOT_DEPLOYED,
};
use multiversx_sc::{
    imports::OptionalValue,
    types::{BigUint, ManagedBuffer, ManagedVec},
};
use multiversx_sc_scenario::ScenarioTxWhitebox;
use proxies::sovereign_forge_proxy::ScArray;
use sovereign_forge::common::{
    storage::StorageModule,
    utils::{ScArray as ScArrayFromUtils, UtilsModule},
};
use sovereign_forge_blackbox_setup::SovereignForgeTestState;
use structs::configs::{EsdtSafeConfig, SovereignConfig};
mod sovereign_forge_blackbox_setup;

/// ### TEST
/// S-FORGE_DEPLOY_OK_001
///
/// ### ACTION
/// Deploy sovereign_forge and chain_factory
///
/// ### EXPECTED
/// Both sovereign_forge and chain_factory contracts deploy successfully
#[test]
fn test_deploy_contracts() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();
    state.common_setup.deploy_chain_factory();
}

/// ### TEST
/// S-FORGE_REGISTER_TOKEN_HANDLER_OK_002
///
/// ### ACTION
/// Register token handler for any shard
///
/// ### EXPECTED
/// sovereign_forge.token_handlers() storage is non-empty
#[test]
fn test_register_token_handler() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();

    state.register_token_handler(2, CHAIN_FACTORY_SC_ADDRESS, None);

    state
        .common_setup
        .world
        .query()
        .to(SOVEREIGN_FORGE_SC_ADDRESS)
        .whitebox(sovereign_forge::contract_obj, |sc| {
            assert!(!sc.token_handlers(2).is_empty());
        });
}

/// ### TEST
/// S-FORGE_REGISTER_CHAIN_FACTORY_OK_003
///
/// ### ACTION
/// Register chain_factory any shard
///
/// ### EXPECTED
/// chain_factories() storage non-empty
#[test]
fn test_register_chain_factory() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();

    state.register_chain_factory(2, CHAIN_FACTORY_SC_ADDRESS, None);

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
/// S-FORGE_UPDATE_CONFIG_FAIL_004
///
/// ### ACTION
/// Update config without deploying chain_config
///
/// ### EXPECTED
/// Error CALLER_DID_NOT_DEPLOY_ANY_SOV_CHAIN
#[test]
fn test_update_sovereign_config_no_chain_config_deployed() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();

    state.register_chain_factory(2, CHAIN_FACTORY_SC_ADDRESS, None);

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
/// S-FORGE_UPDATE_CONFIG_OK_005
///
/// ### ACTION
/// Update sovereign config
///
/// ### EXPECTED
/// Sovereign config was modified
#[test]
fn test_update_sovereign_config() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();
    state.common_setup.deploy_chain_factory();
    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);
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
        &SovereignConfig::default_config(),
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

            let is_chain_config_deployed = sc.is_contract_deployed(
                &OWNER_ADDRESS.to_managed_address(),
                ScArrayFromUtils::ChainConfig,
            );
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
/// S-FORGE_UPDATE_ESDT_SAFE_CONFIG_OK_006
///
/// ### ACTION
/// Update ESDT safe config
///
/// ### EXPECTED
/// ESDT safe config was modified
#[test]
fn test_update_esdt_safe_config() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();
    state.common_setup.deploy_chain_factory();
    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);
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
        &SovereignConfig::default_config(),
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

            let is_chain_config_deployed = sc.is_contract_deployed(
                &OWNER_ADDRESS.to_managed_address(),
                ScArrayFromUtils::ChainConfig,
            );
            assert!(is_chain_config_deployed);
        });

    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);
    state
        .common_setup
        .deploy_mvx_esdt_safe(HEADER_VERIFIER_ADDRESS, OptionalValue::None);

    state.common_setup.deploy_phase_two(None);
    state
        .common_setup
        .world
        .query()
        .to(SOVEREIGN_FORGE_SC_ADDRESS)
        .whitebox(sovereign_forge::contract_obj, |sc| {
            let is_header_verifier_deployed = sc.is_contract_deployed(
                &OWNER_ADDRESS.to_managed_address(),
                ScArrayFromUtils::HeaderVerifier,
            );

            assert!(is_header_verifier_deployed);
        });

    state
        .common_setup
        .deploy_phase_three(OptionalValue::None, None);
    state
        .common_setup
        .world
        .query()
        .to(SOVEREIGN_FORGE_SC_ADDRESS)
        .whitebox(sovereign_forge::contract_obj, |sc| {
            let is_esdt_safe_deployed = sc.is_contract_deployed(
                &OWNER_ADDRESS.to_managed_address(),
                ScArrayFromUtils::ESDTSafe,
            );

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
/// S-FORGE_COMPLETE_SETUP_PHASE_OK_007
///
/// ### ACTION
/// Run deploy phases 1–4 and call complete_setup_phase
///
/// ### EXPECTED
/// Setup phase is complete
#[test]
fn test_complete_setup_phase() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();
    state.common_setup.deploy_chain_factory();
    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);
    state.finish_setup();

    let deploy_cost = BigUint::from(DEPLOY_COST);
    let preffered_chain_id = ManagedBuffer::from(CHAIN_ID);
    state.common_setup.deploy_phase_one(
        &deploy_cost,
        Some(preffered_chain_id.clone()),
        &SovereignConfig::default_config(),
        None,
    );

    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);
    state
        .common_setup
        .deploy_mvx_esdt_safe(HEADER_VERIFIER_ADDRESS, OptionalValue::None);

    state.common_setup.deploy_phase_two(None);
    state
        .common_setup
        .deploy_phase_three(OptionalValue::None, None);
    state.common_setup.deploy_phase_four(None, None);

    state
        .common_setup
        .world
        .query()
        .to(SOVEREIGN_FORGE_SC_ADDRESS)
        .whitebox(sovereign_forge::contract_obj, |sc| {
            let is_chain_config_deployed = sc.is_contract_deployed(
                &OWNER_ADDRESS.to_managed_address(),
                ScArrayFromUtils::ChainConfig,
            );
            let is_header_verifier_deployed = sc.is_contract_deployed(
                &OWNER_ADDRESS.to_managed_address(),
                ScArrayFromUtils::HeaderVerifier,
            );
            let is_esdt_safe_deployed = sc.is_contract_deployed(
                &OWNER_ADDRESS.to_managed_address(),
                ScArrayFromUtils::ESDTSafe,
            );
            let is_fee_market_deployed = sc.is_contract_deployed(
                &OWNER_ADDRESS.to_managed_address(),
                ScArrayFromUtils::FeeMarket,
            );

            assert!(
                is_chain_config_deployed
                    && is_header_verifier_deployed
                    && is_esdt_safe_deployed
                    && is_fee_market_deployed
            );
        });

    state.complete_setup_phase(None);
    state.check_setup_phase_completed(preffered_chain_id, true);
}

/// ### TEST
/// S-FORGE_DEPLOY_PHASE_ONE_FAIL_008
///
/// ### ACTION
/// deploy_phase_one with insufficient cost
///
/// ### EXPECTED
/// Error DEPLOY_COST_NOT_ENOUGH
#[test]
fn test_deploy_phase_one_deploy_cost_too_low() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();
    state.common_setup.deploy_chain_factory();
    state.finish_setup();

    let deploy_cost = BigUint::from(1u32);

    state.common_setup.deploy_phase_one(
        &deploy_cost,
        None,
        &SovereignConfig::default_config(),
        Some(DEPLOY_COST_NOT_ENOUGH),
    );
}

/// ### TEST
/// S-FORGE_DEPLOY_PHASE_ONE_FAIL_009
///
/// ### ACTION
/// Call deploy_phase_one twice for same chain_config
///
/// ### EXPECTED
/// Error CHAIN_CONFIG_ALREADY_DEPLOYED
#[test]
fn test_deploy_phase_one_chain_config_already_deployed() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();
    state.common_setup.deploy_chain_factory();
    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state.finish_setup();

    let deploy_cost = BigUint::from(DEPLOY_COST);
    let config = SovereignConfig::default_config();

    state
        .common_setup
        .deploy_phase_one(&deploy_cost, None, &config, None);
    state.common_setup.deploy_phase_one(
        &deploy_cost,
        None,
        &config,
        Some(CHAIN_CONFIG_ALREADY_DEPLOYED),
    );
}

/// ### TEST
/// S-FORGE_DEPLOY_PHASE_ONE_FAIL_010
///
/// ### ACTION
/// Call deploy_phase_one wrong chain id format
///
/// ### EXPECTED
/// Error CHAIN_ID_NOT_LOWERCASE_ALPHANUMERIC
#[test]
fn test_deploy_phase_one_preferred_chain_id_not_lowercase_alphanumeric() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();
    state.common_setup.deploy_chain_factory();
    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state.finish_setup();

    let deploy_cost = BigUint::from(DEPLOY_COST);

    state.common_setup.deploy_phase_one(
        &deploy_cost,
        Some(ManagedBuffer::from("CHID")),
        &SovereignConfig::default_config(),
        Some(CHAIN_ID_NOT_LOWERCASE_ALPHANUMERIC),
    );
}

/// ### TEST
/// S-FORGE_DEPLOY_PHASE_ONE_FAIL_011
///
/// ### ACTION
/// Call deploy_phase_one wrong chain id length
///
/// ### EXPECTED
/// Error CHAIN_ID_NOT_FOUR_CHAR_LONG
#[test]
fn test_deploy_phase_one_preferred_chain_id_not_correct_length() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();
    state.common_setup.deploy_chain_factory();
    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state.finish_setup();

    let deploy_cost = BigUint::from(DEPLOY_COST);

    state.common_setup.deploy_phase_one(
        &deploy_cost,
        Some(ManagedBuffer::from("CHAINID")),
        &SovereignConfig::default_config(),
        Some(CHAIN_ID_NOT_FOUR_CHAR_LONG),
    );
}

/// ### TEST
/// S-FORGE_DEPLOY_PHASE_ONE_OK_012
///
/// ### ACTION
/// Call deploy_phase_one with no preferred chain id
///
/// ### EXPECTED
/// Chain-Config is deployed and address is set in storage
#[test]
fn test_deploy_phase_one_no_preferred_chain_id() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();
    state.common_setup.deploy_chain_factory();
    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state.finish_setup();

    let deploy_cost = BigUint::from(DEPLOY_COST);

    state.common_setup.deploy_phase_one(
        &deploy_cost,
        None,
        &SovereignConfig::default_config(),
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

            let is_chain_config_deployed = sc.is_contract_deployed(
                &OWNER_ADDRESS.to_managed_address(),
                ScArrayFromUtils::ChainConfig,
            );
            assert!(is_chain_config_deployed);
        })
}

/// ### TEST
/// S-FORGE_DEPLOY_PHASE_ONE_OK_013
///
/// ### ACTION
/// Call deploy_phase_one with preferred chain id
///
/// ### EXPECTED
/// Chain-Config is deployed and address is set in storage
#[test]
fn test_deploy_phase_one_preferred_chain_id() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();
    state.common_setup.deploy_chain_factory();
    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state.finish_setup();

    let deploy_cost = BigUint::from(DEPLOY_COST);

    state.common_setup.deploy_phase_one(
        &deploy_cost,
        Some(ManagedBuffer::from(CHAIN_ID)),
        &SovereignConfig::default_config(),
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

            let is_chain_config_deployed = sc.is_contract_deployed(
                &OWNER_ADDRESS.to_managed_address(),
                ScArrayFromUtils::ChainConfig,
            );
            assert!(is_chain_config_deployed);
        })
}

/// ### TEST
/// S-FORGE_DEPLOY_PHASE_ONE_FAIL_014
///
/// ### ACTION
/// Call deploy_phase_one with an used chain id
///
/// ### EXPECTED
/// Error CHAIN_ID_ALREADY_IN_USE
#[test]
fn test_deploy_phase_one_with_chain_id_used() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();
    state.common_setup.deploy_chain_factory();
    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state.finish_setup();

    let deploy_cost = BigUint::from(DEPLOY_COST);

    state.common_setup.deploy_phase_one(
        &deploy_cost,
        Some(ManagedBuffer::from(CHAIN_ID)),
        &SovereignConfig::default_config(),
        None,
    );

    state.common_setup.deploy_phase_one(
        &deploy_cost,
        Some(ManagedBuffer::from(CHAIN_ID)),
        &SovereignConfig::default_config(),
        Some(CHAIN_ID_ALREADY_IN_USE),
    );
}

/// ### TEST
/// S-FORGE_DEPLOY_PHASE_TWO_FAIL_015
///
/// ### ACTION
/// Call deploy_phase_two without the first phase
///
/// ### EXPECTED
/// Error CALLER_DID_NOT_DEPLOY_ANY_SOV_CHAIN
#[test]
fn test_deploy_phase_two_without_first_phase() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();
    state.common_setup.deploy_chain_factory();
    state.finish_setup();

    state
        .common_setup
        .deploy_phase_two(Some(CALLER_DID_NOT_DEPLOY_ANY_SOV_CHAIN));
}

/// ### TEST
/// S-FORGE_DEPLOY_PHASE_TWO_OK_016
///
/// ### ACTION
/// Call deploy_phase_two
///
/// ### EXPECTED
/// Header-Verifier is deployed and address is set in the storage
#[test]
fn test_deploy_phase_two() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();
    state.common_setup.deploy_chain_factory();
    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state.finish_setup();

    let deploy_cost = BigUint::from(DEPLOY_COST);

    state.common_setup.deploy_phase_one(
        &deploy_cost,
        None,
        &SovereignConfig::default_config(),
        None,
    );
    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);

    state.common_setup.deploy_phase_two(None);

    state
        .common_setup
        .world
        .query()
        .to(SOVEREIGN_FORGE_SC_ADDRESS)
        .whitebox(sovereign_forge::contract_obj, |sc| {
            let is_header_verifier_deployed = sc.is_contract_deployed(
                &OWNER_ADDRESS.to_managed_address(),
                ScArrayFromUtils::HeaderVerifier,
            );

            assert!(is_header_verifier_deployed);
        })
}

/// ### TEST
/// S-FORGE_DEPLOY_PHASE_TWO_FAIL_017
///
/// ### ACTION
/// Call deploy_phase_two two times
///
/// ### EXPECTED
/// Error HEADER_VERIFIER_ALREADY_DEPLOYED
#[test]
fn test_deploy_phase_two_header_already_deployed() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();
    state.common_setup.deploy_chain_factory();
    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state.finish_setup();

    let deploy_cost = BigUint::from(DEPLOY_COST);

    state.common_setup.deploy_phase_one(
        &deploy_cost,
        None,
        &SovereignConfig::default_config(),
        None,
    );
    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);

    state.common_setup.deploy_phase_two(None);
    state
        .common_setup
        .deploy_phase_two(Some(HEADER_VERIFIER_ALREADY_DEPLOYED));
}

/// ### TEST
/// S-FORGE_DEPLOY_PHASE_THREE_OK_018
///
/// ### ACTION
/// Call deploy_phase_three
///
/// ### EXPECTED
/// Mvx-ESDT-Safe is deployed and address is set in storage
#[test]
fn test_deploy_phase_three() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();
    state.common_setup.deploy_chain_factory();
    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state.finish_setup();

    let deploy_cost = BigUint::from(DEPLOY_COST);

    state.common_setup.deploy_phase_one(
        &deploy_cost,
        None,
        &SovereignConfig::default_config(),
        None,
    );
    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);
    state
        .common_setup
        .deploy_mvx_esdt_safe(HEADER_VERIFIER_ADDRESS, OptionalValue::None);

    state.common_setup.deploy_phase_two(None);
    state
        .common_setup
        .deploy_phase_three(OptionalValue::None, None);

    state
        .common_setup
        .world
        .query()
        .to(SOVEREIGN_FORGE_SC_ADDRESS)
        .whitebox(sovereign_forge::contract_obj, |sc| {
            let is_esdt_safe_deployed = sc.is_contract_deployed(
                &OWNER_ADDRESS.to_managed_address(),
                ScArrayFromUtils::ESDTSafe,
            );

            assert!(is_esdt_safe_deployed);
        })
}

/// ### TEST
/// S-FORGE_DEPLOY_PHASE_THREE_FAIL_019
///
/// ### ACTION
/// Call deploy_phase_three without the phase one
///
/// ### EXPECTED
/// Error HEADER_VERIFIER_NOT_DEPLOYED
#[test]
fn test_deploy_phase_three_without_phase_one() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();
    state.common_setup.deploy_chain_factory();
    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state.finish_setup();

    state
        .common_setup
        .deploy_phase_three(OptionalValue::None, Some(HEADER_VERIFIER_NOT_DEPLOYED));
}

/// ### TEST
/// S-FORGE_DEPLOY_PHASE_THREE_FAIL_020
///
/// ### ACTION
/// Call deploy_phase_three without the phase two
///
/// ### EXPECTED
/// Error HEADER_VERIFIER_NOT_DEPLOYED
#[test]
fn test_deploy_phase_three_without_phase_two() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();
    state.common_setup.deploy_chain_factory();
    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state.finish_setup();

    let deploy_cost = BigUint::from(DEPLOY_COST);
    state.common_setup.deploy_phase_one(
        &deploy_cost,
        None,
        &SovereignConfig::default_config(),
        None,
    );

    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);
    state
        .common_setup
        .deploy_mvx_esdt_safe(HEADER_VERIFIER_ADDRESS, OptionalValue::None);

    state
        .common_setup
        .deploy_phase_three(OptionalValue::None, Some(HEADER_VERIFIER_NOT_DEPLOYED));
}

/// ### TEST
/// S-FORGE_DEPLOY_PHASE_THREE_FAIL_021
///
/// ### ACTION
/// Call deploy_phase_three two times
///
/// ### EXPECTED
/// Error ESDT_SAFE_ALREADY_DEPLOYED
#[test]
fn test_deploy_phase_three_already_deployed() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();
    state.common_setup.deploy_chain_factory();
    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state.finish_setup();

    let deploy_cost = BigUint::from(DEPLOY_COST);
    state.common_setup.deploy_phase_one(
        &deploy_cost,
        None,
        &SovereignConfig::default_config(),
        None,
    );

    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);
    state
        .common_setup
        .deploy_mvx_esdt_safe(HEADER_VERIFIER_ADDRESS, OptionalValue::None);

    state.common_setup.deploy_phase_two(None);
    state
        .common_setup
        .deploy_phase_three(OptionalValue::None, None);
    state
        .common_setup
        .deploy_phase_three(OptionalValue::None, Some(ESDT_SAFE_ALREADY_DEPLOYED));
}

/// ### TEST
/// S-FORGE_COMPLETE_SETUP_PHASE_FAIL_022
///
/// ### ACTION
/// Call complete_setup_phase without phase four deployed
///
/// ### EXPECTED
/// Error FEE_MARKET_NOT_DEPLOYED
#[test]
fn test_complete_setup_phase_four_not_deployed() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();
    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);
    state.complete_setup_phase(Some(FEE_MARKET_NOT_DEPLOYED));
}

/// ### TEST
/// S-FORGE_DEPLOY_PHASE_FOUR_OK_023
///
/// ### ACTION
/// Call deploy_phase_four
///
/// ### EXPECTED
/// Fee-Market is deployed and address is set in storage
#[test]
fn test_deploy_phase_four() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();
    state.common_setup.deploy_chain_factory();
    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);
    state.finish_setup();

    let deploy_cost = BigUint::from(DEPLOY_COST);
    state.common_setup.deploy_phase_one(
        &deploy_cost,
        None,
        &SovereignConfig::default_config(),
        None,
    );

    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);
    state
        .common_setup
        .deploy_mvx_esdt_safe(HEADER_VERIFIER_ADDRESS, OptionalValue::None);

    state.common_setup.deploy_phase_two(None);
    state
        .common_setup
        .deploy_phase_three(OptionalValue::None, None);
    state.common_setup.deploy_phase_four(None, None);

    state
        .common_setup
        .world
        .query()
        .to(SOVEREIGN_FORGE_SC_ADDRESS)
        .whitebox(sovereign_forge::contract_obj, |sc| {
            let is_fee_market_deployed = sc.is_contract_deployed(
                &OWNER_ADDRESS.to_managed_address(),
                ScArrayFromUtils::FeeMarket,
            );

            assert!(is_fee_market_deployed);
        })
}

/// ### TEST
/// S-FORGE_DEPLOY_PHASE_FOUR_FAIL_024
///
/// ### ACTION
/// Call deploy_phase_four without phase three
///
/// ### EXPECTED
/// Error ESDT_SAFE_NOT_DEPLOYED
#[test]
fn test_deploy_phase_four_without_previous_phase() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();
    state.common_setup.deploy_chain_factory();
    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);
    state.finish_setup();

    let deploy_cost = BigUint::from(DEPLOY_COST);
    state.common_setup.deploy_phase_one(
        &deploy_cost,
        None,
        &SovereignConfig::default_config(),
        None,
    );

    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);
    state
        .common_setup
        .deploy_mvx_esdt_safe(HEADER_VERIFIER_ADDRESS, OptionalValue::None);

    state.common_setup.deploy_phase_two(None);
    state
        .common_setup
        .deploy_phase_four(None, Some(ESDT_SAFE_NOT_DEPLOYED));
}

/// ### TEST
/// S-FORGE_DEPLOY_PHASE_FOUR_FAIL_025
///
/// ### ACTION
/// Call deploy_phase_four two times
///
/// ### EXPECTED
/// Error FEE_MARKET_ALREADY_DEPLOYED
#[test]
fn test_deploy_phase_four_fee_market_already_deployed() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();
    state.common_setup.deploy_chain_factory();
    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);
    state.finish_setup();

    let deploy_cost = BigUint::from(DEPLOY_COST);
    state.common_setup.deploy_phase_one(
        &deploy_cost,
        None,
        &SovereignConfig::default_config(),
        None,
    );

    state
        .common_setup
        .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);
    state
        .common_setup
        .deploy_mvx_esdt_safe(HEADER_VERIFIER_ADDRESS, OptionalValue::None);

    state.common_setup.deploy_phase_two(None);
    state
        .common_setup
        .deploy_phase_three(OptionalValue::None, None);
    state.common_setup.deploy_phase_four(None, None);
    state
        .common_setup
        .deploy_phase_four(None, Some(FEE_MARKET_ALREADY_DEPLOYED));
}
