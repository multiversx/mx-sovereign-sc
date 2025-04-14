use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use multiversx_sc_snippets::multiversx_sc_scenario::multiversx_chain_vm::crypto_functions::sha256;
use multiversx_sc_snippets::multiversx_sc_scenario::scenario_model::{Log, TxResponseStatus};
use multiversx_sc_snippets::sdk::gateway::SetStateAccount;
use multiversx_sc_snippets::{hex, imports::*};
use proxies::chain_config_proxy::ChainConfigContractProxy;
use proxies::fee_market_proxy::{FeeMarketProxy, FeeStruct};
use proxies::header_verifier_proxy::HeaderverifierProxy;
use proxies::mvx_esdt_safe_proxy::MvxEsdtSafeProxy;
use proxies::testing_sc_proxy::TestingScProxy;
use structs::aliases::{OptionalValueTransferDataTuple, PaymentsVec};

use structs::configs::{EsdtSafeConfig, SovereignConfig};
use structs::operation::Operation;

use crate::{config::Config, State};
use common_blackbox_setup::{
    RegisterTokenArgs, CHAIN_CONFIG_CODE_PATH, FEE_MARKET_CODE_PATH, HEADER_VERIFIER_CODE_PATH,
    MVX_ESDT_SAFE_CODE_PATH, TESTING_SC_CODE_PATH,
};

pub struct MvxEsdtSafeInteract {
    pub interactor: Interactor,
    pub owner_address: Address,
    pub user_address: Address,
    pub state: State,
}

impl MvxEsdtSafeInteract {
    pub async fn new(config: Config) -> Self {
        let mut interactor = Interactor::new(config.gateway_uri())
            .await
            .use_chain_simulator(config.use_chain_simulator());

        interactor.set_current_dir_from_workspace("interactor");
        let owner_address = interactor.register_wallet(test_wallets::mike()).await;
        let user_address = interactor.register_wallet(test_wallets::bob()).await;

        // Useful in the chain simulator setting
        // generate blocks until ESDTSystemSCAddress is enabled
        interactor.generate_blocks_until_epoch(1u64).await.unwrap();

        let set_state_response = interactor.set_state_for_saved_accounts().await;
        interactor.generate_blocks(2u64).await.unwrap();
        assert!(set_state_response.is_ok());

        MvxEsdtSafeInteract {
            interactor,
            owner_address,
            user_address,
            state: State::load_state(),
        }
    }

    pub fn assert_expected_log(&mut self, logs: Vec<Log>, expected_log: &str) {
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

    // Key and value should be in hex
    pub async fn check_account_storage(
        &mut self,
        address: Address,
        wanted_key: &str,
        expected_value: Option<&str>,
    ) {
        let pairs = self.interactor.get_account_storage(&address).await;

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

    pub fn decode_from_hex(&mut self, hex_string: &str) -> String {
        let bytes =
            hex::decode(hex_string).expect("Failed to decode hex string: invalid hex format");
        String::from_utf8(bytes).expect("Failed to decode UTF-8 string: invalid UTF-8 bytes")
    }

    pub async fn deploy_mvx_esdt_safe(
        &mut self,
        header_verifier_address: Bech32Address,
        opt_config: OptionalValue<EsdtSafeConfig<StaticApi>>,
    ) {
        let new_address = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .init(header_verifier_address, opt_config)
            .code(MVX_ESDT_SAFE_CODE_PATH)
            .code_metadata(CodeMetadata::all())
            .returns(ReturnsNewAddress)
            .run()
            .await;

        let new_address_bech32 = bech32::encode(&new_address);
        self.state
            .set_mvx_esdt_safe_contract_address(Bech32Address::from_bech32_string(
                new_address_bech32.clone(),
            ));

        println!("new mvx esdt safe address: {new_address_bech32}");
    }

    pub async fn deploy_header_verifier(&mut self) {
        let new_address = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .gas(30_000_000u64)
            .typed(HeaderverifierProxy)
            .init()
            .code(HEADER_VERIFIER_CODE_PATH)
            .code_metadata(CodeMetadata::all())
            .returns(ReturnsNewAddress)
            .run()
            .await;

        let new_address_bech32 = bech32::encode(&new_address);
        self.state
            .set_header_verifier_address(Bech32Address::from_bech32_string(
                new_address_bech32.clone(),
            ));

        println!("new header verifier address: {new_address_bech32}");
    }

    pub async fn deploy_fee_market(
        &mut self,
        esdt_safe_address: Bech32Address,
        fee: Option<FeeStruct<StaticApi>>,
    ) {
        let new_address = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .gas(60_000_000u64)
            .typed(FeeMarketProxy)
            .init(esdt_safe_address, fee)
            .code(FEE_MARKET_CODE_PATH)
            .code_metadata(CodeMetadata::all())
            .returns(ReturnsNewAddress)
            .run()
            .await;

        let new_address_bech32 = bech32::encode(&new_address);
        self.state
            .set_fee_market_address(Bech32Address::from_bech32_string(
                new_address_bech32.clone(),
            ));

        println!("new fee market address: {new_address_bech32}");
    }

    pub async fn deploy_testing_sc(&mut self) {
        let new_address = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .gas(30_000_000u64)
            .typed(TestingScProxy)
            .init()
            .code(TESTING_SC_CODE_PATH)
            .code_metadata(CodeMetadata::all())
            .returns(ReturnsNewAddress)
            .run()
            .await;

        let new_address_bech32 = bech32::encode(&new_address);
        self.state
            .set_testing_sc_address(Bech32Address::from_bech32_string(
                new_address_bech32.clone(),
            ));

        println!("new testing sc address: {new_address_bech32}");
    }

    pub async fn set_esdt_safe_address_in_header_verifier(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .to(self.state.current_header_verifier_address())
            .gas(30_000_000u64)
            .typed(HeaderverifierProxy)
            .set_esdt_safe_address(self.state.current_mvx_esdt_safe_contract_address())
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn deploy_chain_config(&mut self) {
        let config = SovereignConfig::default_config();
        let new_address = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .gas(60_000_000u64)
            .typed(ChainConfigContractProxy)
            .init(
                config.min_validators as usize,
                config.max_validators as usize,
                config.min_stake,
                self.owner_address.clone(),
                MultiValueEncoded::new(),
            )
            .code(CHAIN_CONFIG_CODE_PATH)
            .code_metadata(CodeMetadata::all())
            .returns(ReturnsNewAddress)
            .run()
            .await;

        let new_address_bech32 = bech32::encode(&new_address);
        self.state
            .set_chain_config_sc_address(Bech32Address::from_bech32_string(
                new_address_bech32.clone(),
            ));

        println!("new chain config sc address: {new_address_bech32}");
    }

    pub async fn deploy_contracts(
        &mut self,
        esdt_safe_config: OptionalValue<EsdtSafeConfig<StaticApi>>,
        fee_struct: Option<FeeStruct<StaticApi>>,
    ) {
        self.deploy_header_verifier().await;
        self.deploy_mvx_esdt_safe(
            self.state.current_header_verifier_address().clone(),
            esdt_safe_config,
        )
        .await;
        self.deploy_fee_market(
            self.state.current_mvx_esdt_safe_contract_address().clone(),
            fee_struct,
        )
        .await;
        self.set_fee_market_address(self.state.current_fee_market_address().to_address())
            .await;
        self.unpause_endpoint().await;
    }

    pub async fn register_operation(
        &mut self,
        signature: ManagedBuffer<StaticApi>,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        operations_hashes: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>>,
    ) {
        self.interactor
            .tx()
            .from(&self.owner_address)
            .to(self.state.current_header_verifier_address())
            .gas(30_000_000u64)
            .typed(HeaderverifierProxy)
            .register_bridge_operations(
                signature,
                hash_of_hashes,
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                operations_hashes,
            )
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
    }

    pub async fn upgrade(&mut self) {
        let response = self
            .interactor
            .tx()
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .from(&self.owner_address)
            .gas(30_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .upgrade()
            .code(MVX_ESDT_SAFE_CODE_PATH)
            .code_metadata(CodeMetadata::UPGRADEABLE)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn update_configuration(&mut self, new_config: EsdtSafeConfig<StaticApi>) {
        let response = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .gas(30_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .update_configuration(new_config)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn set_fee_market_address(&mut self, fee_market_address: Address) {
        let response = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .gas(30_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .set_fee_market_address(fee_market_address)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn deposit(
        &mut self,
        to: Address,
        opt_transfer_data: OptionalValueTransferDataTuple<StaticApi>,
        payments: PaymentsVec<StaticApi>,
        expected_error_message: Option<&str>,
        expected_log: Option<&str>,
    ) {
        let (response, logs) = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .deposit(to, opt_transfer_data)
            .payment(payments)
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run()
            .await;

        self.assert_expected_error_message(response, expected_error_message);

        if let Some(expected_log) = expected_log {
            self.assert_expected_log(logs, expected_log);
        }
    }

    pub async fn execute_operations(
        &mut self,
        hash_of_hashes: ManagedBuffer<StaticApi>,
        operation: Operation<StaticApi>,
        expected_error_message: Option<&str>,
        expected_log: Option<&str>,
    ) {
        let (response, logs) = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .gas(120_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .execute_operations(hash_of_hashes, operation)
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run()
            .await;

        self.assert_expected_error_message(response, expected_error_message);

        if let Some(expected_log) = expected_log {
            self.assert_expected_log(logs, expected_log);
        }
    }

    pub async fn register_token(
        &mut self,
        args: RegisterTokenArgs<'_>,
        egld_amount: BigUint<StaticApi>,
        expected_error_message: Option<&str>,
    ) {
        let response = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .register_token(
                args.sov_token_id,
                args.token_type,
                args.token_display_name,
                args.token_ticker,
                args.num_decimals,
            )
            .egld(egld_amount)
            .returns(ReturnsHandledOrError::new())
            .run()
            .await;

        self.assert_expected_error_message(response, expected_error_message);
    }

    pub async fn pause_endpoint(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .gas(30_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .pause_endpoint()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn unpause_endpoint(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .gas(30_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .unpause_endpoint()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn paused_status(&mut self) {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .typed(MvxEsdtSafeProxy)
            .paused_status()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    pub async fn set_max_bridged_amount(&mut self) {
        let token_id = TokenIdentifier::from_esdt_bytes(&b""[..]);
        let max_amount = BigUint::<StaticApi>::from(0u128);

        let response = self
            .interactor
            .tx()
            .from(&self.owner_address)
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .gas(30_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .set_max_bridged_amount(token_id, max_amount)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn max_bridged_amount(&mut self) {
        let token_id = TokenIdentifier::from_esdt_bytes(&b""[..]);

        let result_value = self
            .interactor
            .query()
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .typed(MvxEsdtSafeProxy)
            .max_bridged_amount(token_id)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {result_value:?}");
    }

    //TODO: Make this a common function in common-blackbox-setup
    pub fn get_operation_hash(
        &mut self,
        operation: &Operation<StaticApi>,
    ) -> ManagedBuffer<StaticApi> {
        let mut serialized_operation: ManagedBuffer<StaticApi> = ManagedBuffer::new();
        let _ = operation.top_encode(&mut serialized_operation);
        let sha256 = sha256(&serialized_operation.to_vec());

        ManagedBuffer::new_from_bytes(&sha256)
    }

    pub async fn reset_state_chain_sim(&mut self, address_states: Option<Vec<Bech32Address>>) {
        let mut state_vec = vec![
            SetStateAccount::from_address(
                Bech32Address::from(self.owner_address.clone()).to_bech32_string(),
            ),
            SetStateAccount::from_address(
                Bech32Address::from(self.user_address.clone()).to_bech32_string(),
            ),
            SetStateAccount::from_address(
                self.state
                    .current_mvx_esdt_safe_contract_address()
                    .to_bech32_string(),
            ),
            SetStateAccount::from_address(
                self.state
                    .current_header_verifier_address()
                    .to_bech32_string(),
            ),
        ];

        if let Some(address_states) = address_states {
            for address in address_states {
                state_vec.push(SetStateAccount::from_address(address.to_bech32_string()));
            }
        }
        let response = self.interactor.set_state_overwrite(state_vec).await;
        self.interactor.generate_blocks(2u64).await.unwrap();
        assert!(response.is_ok());
    }
}
