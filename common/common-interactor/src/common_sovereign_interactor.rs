#![allow(async_fn_in_trait)]

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use common_test_setup::constants::{
    CHAIN_CONFIG_CODE_PATH, CHAIN_FACTORY_CODE_PATH, ENSHRINE_ESDT_SAFE_CODE_PATH,
    FEE_MARKET_CODE_PATH, HEADER_VERIFIER_CODE_PATH, ISSUE_COST, MVX_ESDT_SAFE_CODE_PATH,
    SOVEREIGN_FORGE_CODE_PATH, TESTING_SC_CODE_PATH, TOKEN_HANDLER_CODE_PATH,
};
use multiversx_sc::{
    codec::TopEncode,
    imports::{ESDTSystemSCProxy, OptionalValue},
    types::{
        Address, BigUint, CodeMetadata, ESDTSystemSCAddress, ManagedBuffer, ReturnsNewAddress,
        ReturnsResultUnmanaged, TokenIdentifier,
    },
};
use multiversx_sc_snippets::{
    hex,
    imports::{bech32, Bech32Address, ReturnsNewTokenIdentifier, StaticApi},
    multiversx_sc_scenario::{
        multiversx_chain_vm::crypto_functions::sha256,
        scenario_model::{Log, TxResponseStatus},
    },
    Interactor, InteractorRunAsync,
};
use proxies::{
    chain_config_proxy::ChainConfigContractProxy, chain_factory_proxy::ChainFactoryContractProxy,
    enshrine_esdt_safe_proxy, fee_market_proxy::FeeMarketProxy,
    header_verifier_proxy::HeaderverifierProxy, mvx_esdt_safe_proxy::MvxEsdtSafeProxy,
    sovereign_forge_proxy::SovereignForgeProxy, testing_sc_proxy::TestingScProxy,
    token_handler_proxy,
};
use structs::{
    configs::{EsdtSafeConfig, SovereignConfig},
    fee::FeeStruct,
    operation::Operation,
};

use crate::{
    interactor_enums::{EsdtTokenProperties, IssueTokenStruct},
    interactor_state::State,
};

pub trait CommonInteractorTrait {
    fn interactor(&mut self) -> &mut Interactor;
    fn state(&mut self) -> &mut State;
    fn wallet_address(&mut self) -> &Address;

    async fn issue_token(&mut self, token_struct: IssueTokenStruct) -> String {
        let wallet_address = self.wallet_address().clone();
        let interactor = self.interactor();
        let base_tx = interactor
            .tx()
            .from(wallet_address.clone())
            .to(ESDTSystemSCAddress)
            .gas(100_000_000u64)
            .typed(ESDTSystemSCProxy);

        match token_struct {
            IssueTokenStruct::Fungible {
                token_display_name,
                token_ticker,
                initial_supply,
                properties,
            } => {
                let attributes = match properties {
                    EsdtTokenProperties::Fungible(props) => props,
                    _ => panic!("Invalid token properties for Fungible token"),
                };
                base_tx
                    .issue_fungible(
                        ISSUE_COST.into(),
                        &token_display_name,
                        &token_ticker,
                        initial_supply,
                        attributes,
                    )
                    .returns(ReturnsNewTokenIdentifier)
                    .run()
                    .await
            }

            IssueTokenStruct::NonFungible {
                token_display_name,
                token_ticker,
                properties,
            } => {
                let attributes = match properties {
                    EsdtTokenProperties::NonFungible(props) => props,
                    _ => panic!("Invalid token properties for NonFungible token"),
                };
                base_tx
                    .issue_non_fungible(
                        ISSUE_COST.into(),
                        &token_display_name,
                        &token_ticker,
                        attributes,
                    )
                    .returns(ReturnsNewTokenIdentifier)
                    .run()
                    .await
            }

            IssueTokenStruct::SemiFungible {
                token_display_name,
                token_ticker,
                properties,
            } => {
                let attributes = match properties {
                    EsdtTokenProperties::SemiFungible(props) => props,
                    _ => panic!("Invalid token properties for SemiFungible token"),
                };
                base_tx
                    .issue_semi_fungible(
                        ISSUE_COST.into(),
                        &token_display_name,
                        &token_ticker,
                        attributes,
                    )
                    .returns(ReturnsNewTokenIdentifier)
                    .run()
                    .await
            }

            IssueTokenStruct::Dynamic {
                token_display_name,
                token_ticker,
                token_type,
                num_decimals,
            } => {
                base_tx
                    .issue_dynamic(
                        ISSUE_COST.into(),
                        &token_display_name,
                        &token_ticker,
                        token_type,
                        num_decimals,
                    )
                    .returns(ReturnsNewTokenIdentifier)
                    .run()
                    .await
            }

            IssueTokenStruct::Meta {
                token_display_name,
                token_ticker,
                properties,
            } => {
                let attributes = match properties {
                    EsdtTokenProperties::Meta(props) => props,
                    _ => panic!("Invalid token properties for Meta token"),
                };
                base_tx
                    .register_meta_esdt(
                        ISSUE_COST.into(),
                        &token_display_name,
                        &token_ticker,
                        attributes,
                    )
                    .returns(ReturnsNewTokenIdentifier)
                    .run()
                    .await
            }
        }
    }

    async fn deploy_sovereign_forge(&mut self, deploy_cost: &BigUint<StaticApi>) {
        let wallet_address = self.wallet_address().clone();

        let new_address = self
            .interactor()
            .tx()
            .from(wallet_address)
            .gas(50_000_000u64)
            .typed(SovereignForgeProxy)
            .init(deploy_cost)
            .code(SOVEREIGN_FORGE_CODE_PATH)
            .code_metadata(CodeMetadata::all())
            .returns(ReturnsNewAddress)
            .run()
            .await;

        let new_address_bech32 = bech32::encode(&new_address);
        self.state()
            .set_sovereign_forge_sc_address(Bech32Address::from_bech32_string(
                new_address_bech32.clone(),
            ));

        println!("new Forge address: {new_address_bech32}");
    }

    async fn deploy_chain_factory(
        &mut self,
        sovereign_forge_address: Bech32Address,
        chain_config_address: Bech32Address,
        header_verifier_address: Bech32Address,
        mvx_esdt_safe_address: Bech32Address,
        fee_market_address: Bech32Address,
    ) {
        let wallet_address = self.wallet_address().clone();

        let new_address = self
            .interactor()
            .tx()
            .from(wallet_address)
            .gas(50_000_000u64)
            .typed(ChainFactoryContractProxy)
            .init(
                sovereign_forge_address,
                chain_config_address,
                header_verifier_address,
                mvx_esdt_safe_address,
                fee_market_address,
            )
            .code(CHAIN_FACTORY_CODE_PATH)
            .code_metadata(CodeMetadata::all())
            .returns(ReturnsNewAddress)
            .run()
            .await;

        let new_address_bech32 = bech32::encode(&new_address);
        self.state()
            .set_chain_factory_sc_address(Bech32Address::from_bech32_string(
                new_address_bech32.clone(),
            ));

        println!("new Chain-Factory address: {new_address_bech32}");
    }

    async fn deploy_chain_config(&mut self, config: SovereignConfig<StaticApi>) {
        let wallet_address = self.wallet_address().clone();

        let new_address = self
            .interactor()
            .tx()
            .from(wallet_address)
            .gas(50_000_000u64)
            .typed(ChainConfigContractProxy)
            .init(config)
            .returns(ReturnsNewAddress)
            .code(CHAIN_CONFIG_CODE_PATH)
            .code_metadata(CodeMetadata::all())
            .run()
            .await;

        let new_address_bech32 = bech32::encode(&new_address);
        self.state()
            .set_chain_config_sc_address(Bech32Address::from_bech32_string(
                new_address_bech32.clone(),
            ));

        println!("new Chain-Config address: {new_address_bech32}");
    }

    async fn deploy_header_verifier(&mut self, chain_config_address: Bech32Address) {
        let wallet_address = self.wallet_address().clone();

        let new_address = self
            .interactor()
            .tx()
            .from(wallet_address)
            .gas(50_000_000u64)
            .typed(HeaderverifierProxy)
            .init(chain_config_address)
            .returns(ReturnsNewAddress)
            .code(HEADER_VERIFIER_CODE_PATH)
            .code_metadata(CodeMetadata::all())
            .run()
            .await;

        let new_address_bech32 = bech32::encode(&new_address);
        self.state()
            .set_header_verifier_address(Bech32Address::from_bech32_string(
                new_address_bech32.clone(),
            ));

        println!("new Header-Verifier address: {new_address_bech32}");
    }

    async fn deploy_mvx_esdt_safe(
        &mut self,
        header_verifier_address: Bech32Address,
        opt_config: OptionalValue<EsdtSafeConfig<StaticApi>>,
    ) {
        let wallet_address = self.wallet_address().clone();

        let new_address = self
            .interactor()
            .tx()
            .from(wallet_address)
            .gas(100_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .init(header_verifier_address, opt_config)
            .returns(ReturnsNewAddress)
            .code(MVX_ESDT_SAFE_CODE_PATH)
            .code_metadata(CodeMetadata::all())
            .run()
            .await;

        let new_address_bech32 = bech32::encode(&new_address);
        self.state()
            .set_mvx_esdt_safe_contract_address(Bech32Address::from_bech32_string(
                new_address_bech32.clone(),
            ));

        println!("new mvx-esdt-safe address: {new_address_bech32}");
    }

    async fn deploy_fee_market(
        &mut self,
        mvx_esdt_safe_address: Bech32Address,
        fee: Option<FeeStruct<StaticApi>>,
    ) {
        let wallet_address = self.wallet_address().clone();

        let new_address = self
            .interactor()
            .tx()
            .from(wallet_address)
            .gas(80_000_000u64)
            .typed(FeeMarketProxy)
            .init(mvx_esdt_safe_address, fee)
            .returns(ReturnsNewAddress)
            .code(FEE_MARKET_CODE_PATH)
            .code_metadata(CodeMetadata::all())
            .run()
            .await;

        let new_address_bech32 = bech32::encode(&new_address);
        self.state()
            .set_fee_market_address(Bech32Address::from_bech32_string(
                new_address_bech32.clone(),
            ));

        println!("new Fee-Market address: {new_address_bech32}");
    }

    async fn deploy_testing_sc(&mut self) {
        let wallet_address = self.wallet_address().clone();

        let new_address = self
            .interactor()
            .tx()
            .from(wallet_address)
            .gas(120_000_000u64)
            .typed(TestingScProxy)
            .init()
            .code(TESTING_SC_CODE_PATH)
            .code_metadata(CodeMetadata::all())
            .returns(ReturnsNewAddress)
            .run()
            .await;

        let new_address_bech32 = bech32::encode(&new_address);
        self.state()
            .set_testing_sc_address(Bech32Address::from_bech32_string(
                new_address_bech32.clone(),
            ));

        println!("new testing sc address: {new_address_bech32}");
    }

    async fn deploy_token_handler(&mut self, chain_factory_address: Bech32Address) {
        let wallet_address = self.wallet_address().clone();

        let new_address = self
            .interactor()
            .tx()
            .from(wallet_address)
            .gas(100_000_000u64)
            .typed(token_handler_proxy::TokenHandlerProxy)
            .init(chain_factory_address)
            .code(TOKEN_HANDLER_CODE_PATH)
            .returns(ReturnsNewAddress)
            .run()
            .await;
        let new_address_bech32 = bech32::encode(&new_address);
        self.state()
            .set_token_handler_address(Bech32Address::from_bech32_string(
                new_address_bech32.clone(),
            ));
        println!("new token_handler_address: {new_address_bech32}");
    }

    async fn deploy_enshrine_esdt(
        &mut self,
        is_sovereign_chain: bool,
        opt_wegld_identifier: Option<TokenIdentifier<StaticApi>>,
        opt_sov_token_prefix: Option<ManagedBuffer<StaticApi>>,
        token_handler_address: Bech32Address,
        opt_config: Option<EsdtSafeConfig<StaticApi>>,
    ) {
        let wallet_address = self.wallet_address().clone();

        let new_address = self
            .interactor()
            .tx()
            .from(wallet_address)
            .gas(100_000_000u64)
            .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
            .init(
                is_sovereign_chain,
                token_handler_address,
                opt_wegld_identifier,
                opt_sov_token_prefix,
                opt_config,
            )
            .code(ENSHRINE_ESDT_SAFE_CODE_PATH)
            .code_metadata(CodeMetadata::all())
            .returns(ReturnsNewAddress)
            .run()
            .await;
        let new_address_bech32 = bech32::encode(&new_address);
        self.state()
            .set_enshrine_esdt_safe_sc_address(Bech32Address::from_bech32_string(
                new_address_bech32.clone(),
            ));

        println!("new address: {new_address_bech32}");
    }

    async fn deploy_phase_one(
        &mut self,
        egld_amount: BigUint<StaticApi>,
        opt_preferred_chain_id: Option<ManagedBuffer<StaticApi>>,
        config: SovereignConfig<StaticApi>,
    ) {
        let wallet_address = self.wallet_address().clone();
        let sovereign_forge_address = self.state().current_sovereign_forge_sc_address().clone();

        let response = self
            .interactor()
            .tx()
            .from(wallet_address)
            .to(sovereign_forge_address)
            .gas(100_000_000u64)
            .typed(SovereignForgeProxy)
            .deploy_phase_one(opt_preferred_chain_id, config)
            .egld(egld_amount)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn deploy_phase_two(&mut self) {
        let wallet_address = self.wallet_address().clone();
        let sovereign_forge_address = self.state().current_sovereign_forge_sc_address().clone();

        let response = self
            .interactor()
            .tx()
            .from(wallet_address)
            .to(sovereign_forge_address)
            .gas(30_000_000u64)
            .typed(SovereignForgeProxy)
            .deploy_phase_two()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn deploy_phase_three(&mut self, opt_config: OptionalValue<EsdtSafeConfig<StaticApi>>) {
        let wallet_address = self.wallet_address().clone();
        let sovereign_forge_address = self.state().current_sovereign_forge_sc_address().clone();

        let response = self
            .interactor()
            .tx()
            .from(wallet_address)
            .to(sovereign_forge_address)
            .gas(80_000_000u64)
            .typed(SovereignForgeProxy)
            .deploy_phase_three(opt_config)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn deploy_phase_four(&mut self, fee: Option<FeeStruct<StaticApi>>) {
        let wallet_address = self.wallet_address().clone();
        let sovereign_forge_address = self.state().current_sovereign_forge_sc_address().clone();

        let response = self
            .interactor()
            .tx()
            .from(wallet_address)
            .to(sovereign_forge_address)
            .gas(80_000_000u64)
            .typed(SovereignForgeProxy)
            .deploy_phase_four(fee)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn complete_setup_phase(&mut self) {
        let wallet_address = self.wallet_address().clone();
        let sovereign_forge_address = self.state().current_sovereign_forge_sc_address().clone();

        let response = self
            .interactor()
            .tx()
            .from(wallet_address)
            .to(sovereign_forge_address)
            .gas(90_000_000u64)
            .typed(SovereignForgeProxy)
            .complete_setup_phase()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn complete_header_verifier_setup_phase(&mut self) {
        let wallet_address = self.wallet_address().clone();
        let header_verifier_address = self.state().current_header_verifier_address().clone();

        self.interactor()
            .tx()
            .from(wallet_address)
            .to(header_verifier_address)
            .gas(90_000_000u64)
            .typed(HeaderverifierProxy)
            .complete_setup_phase()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
    }

    async fn set_esdt_safe_address_in_header_verifier(
        &mut self,
        mvx_esdt_safe_address: Bech32Address,
    ) {
        let wallet_address = self.wallet_address().clone();
        let header_verifier_address = self.state().current_header_verifier_address().clone();

        let response = self
            .interactor()
            .tx()
            .from(wallet_address)
            .to(header_verifier_address)
            .gas(90_000_000u64)
            .typed(HeaderverifierProxy)
            .set_esdt_safe_address(mvx_esdt_safe_address)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    fn assert_expected_log(&mut self, logs: Vec<Log>, expected_log: &str) {
        let expected_bytes = ManagedBuffer::<StaticApi>::from(expected_log).to_vec();

        let found_log = logs.iter().find(|log| {
            log.topics.iter().any(|topic| {
                if let Ok(decoded_topic) = BASE64.decode(topic) {
                    decoded_topic == expected_bytes
                } else {
                    false
                }
            })
        });

        assert!(found_log.is_some(), "Expected log not found");
    }

    fn assert_expected_error_message(
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

    /// Key and value should be in hex
    async fn check_account_storage(
        &mut self,
        address: Address,
        wanted_key: &str,
        expected_value: Option<&str>,
    ) {
        let pairs = self.interactor().get_account_storage(&address).await;

        let found_entry = pairs.iter().find(|(key, _)| key.contains(wanted_key));

        let decoded_key = self.decode_from_hex(wanted_key);

        match expected_value {
            Some(expected) => {
                assert!(
                    found_entry.is_some(),
                    "Expected key containing '{}' (decoded: '{}') was not found in account storage.",
                    wanted_key,
                    decoded_key
                );

                let (_, value) = found_entry.unwrap();

                let decoded_expected = self.decode_from_hex(expected);

                let decoded_value = self.decode_from_hex(value);

                assert!(
                    value.contains(expected),
                    "Mismatch: expected '{}' (decoded: '{}') to be contained in '{}' (decoded: '{}')",
                    expected,
                    decoded_expected,
                    value,
                    decoded_value,
                );
            }
            None => {
                assert!(
                    found_entry.is_none(),
                    "Did not expect to find key containing '{}' (decoded: '{}') in account storage.",
                    wanted_key,
                    decoded_key
                );
            }
        }
    }

    fn decode_from_hex(&mut self, hex_string: &str) -> String {
        let bytes =
            hex::decode(hex_string).expect("Failed to decode hex string: invalid hex format");
        String::from_utf8(bytes).expect("Failed to decode UTF-8 string: invalid UTF-8 bytes")
    }

    fn get_operation_hash(&mut self, operation: &Operation<StaticApi>) -> ManagedBuffer<StaticApi> {
        let mut serialized_operation: ManagedBuffer<StaticApi> = ManagedBuffer::new();
        let _ = operation.top_encode(&mut serialized_operation);
        let sha256 = sha256(&serialized_operation.to_vec());

        ManagedBuffer::new_from_bytes(&sha256)
    }
}
