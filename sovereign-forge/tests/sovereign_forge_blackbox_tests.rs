use chain_config::validator_rules::ValidatorRulesModule;
use common_test_setup::constants::{
    CHAIN_CONFIG_ADDRESS, CHAIN_FACTORY_SC_ADDRESS, CHAIN_ID, ESDT_SAFE_ADDRESS,
    HEADER_VERIFIER_ADDRESS, OWNER_ADDRESS, SOVEREIGN_FORGE_SC_ADDRESS,
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

#[test]
fn test_deploy_contracts() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();
    state.common_setup.deploy_chain_factory();
}

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

    let deploy_cost = BigUint::from(100_000u32);

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

    let deploy_cost = BigUint::from(100_000u32);

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
            100_000,
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
            assert!(max_bridged_amount == 100_000);
        })
}

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

    let deploy_cost = BigUint::from(100_000u32);
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
}

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

#[test]
fn test_deploy_phase_one_chain_config_already_deployed() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();
    state.common_setup.deploy_chain_factory();
    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state.finish_setup();

    let deploy_cost = BigUint::from(100_000u32);
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

#[test]
fn test_deploy_phase_one_preferred_chain_id_not_lowercase_alphanumeric() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();
    state.common_setup.deploy_chain_factory();
    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state.finish_setup();

    let deploy_cost = BigUint::from(100_000u32);

    state.common_setup.deploy_phase_one(
        &deploy_cost,
        Some(ManagedBuffer::from("CHID")),
        &SovereignConfig::default_config(),
        Some(CHAIN_ID_NOT_LOWERCASE_ALPHANUMERIC),
    );
}

#[test]
fn test_deploy_phase_one_preferred_chain_id_not_correct_length() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();
    state.common_setup.deploy_chain_factory();
    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state.finish_setup();

    let deploy_cost = BigUint::from(100_000u32);

    state.common_setup.deploy_phase_one(
        &deploy_cost,
        Some(ManagedBuffer::from("CHAINID")),
        &SovereignConfig::default_config(),
        Some(CHAIN_ID_NOT_FOUR_CHAR_LONG),
    );
}

#[test]
fn test_deploy_phase_one_no_preferred_chain_id() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();
    state.common_setup.deploy_chain_factory();
    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state.finish_setup();

    let deploy_cost = BigUint::from(100_000u32);

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

#[test]
fn test_deploy_phase_one_preferred_chain_id() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();
    state.common_setup.deploy_chain_factory();
    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state.finish_setup();

    let deploy_cost = BigUint::from(100_000u32);

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

#[test]
fn test_deploy_phase_one_with_chain_id_used() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();
    state.common_setup.deploy_chain_factory();
    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state.finish_setup();

    let deploy_cost = BigUint::from(100_000u32);

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

#[test]
fn test_deploy_phase_two() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();
    state.common_setup.deploy_chain_factory();
    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state.finish_setup();

    let deploy_cost = BigUint::from(100_000u32);

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

#[test]
fn test_deploy_phase_two_header_already_deployed() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();
    state.common_setup.deploy_chain_factory();
    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state.finish_setup();

    let deploy_cost = BigUint::from(100_000u32);

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

#[test]
fn test_deploy_phase_three() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();
    state.common_setup.deploy_chain_factory();
    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state.finish_setup();

    let deploy_cost = BigUint::from(100_000u32);

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

#[test]
fn test_deploy_phase_three_without_phase_two() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();
    state.common_setup.deploy_chain_factory();
    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state.finish_setup();

    let deploy_cost = BigUint::from(100_000u32);
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

#[test]
fn test_deploy_phase_three_already_deployed() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();
    state.common_setup.deploy_chain_factory();
    state
        .common_setup
        .deploy_chain_config(SovereignConfig::default_config());
    state.finish_setup();

    let deploy_cost = BigUint::from(100_000u32);
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

#[test]
fn test_complete_setup_phase_four_not_deployed() {
    let mut state = SovereignForgeTestState::new();
    state.common_setup.deploy_sovereign_forge();
    state
        .common_setup
        .deploy_fee_market(None, ESDT_SAFE_ADDRESS);
    state.complete_setup_phase(Some(FEE_MARKET_NOT_DEPLOYED));
}

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

    let deploy_cost = BigUint::from(100_000u32);
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

    let deploy_cost = BigUint::from(100_000u32);
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

    let deploy_cost = BigUint::from(100_000u32);
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
