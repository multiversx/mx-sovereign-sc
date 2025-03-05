use multiversx_sc_snippets::multiversx_sc_scenario::multiversx_chain_vm::crypto_functions::sha256;
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

pub struct MvxEsdtSafeInteract {
    pub interactor: Interactor,
    pub wallet_address: Address,
    pub bob_address: Address,
    mvx_esdt_safe_contract_code: BytesValue,
    header_verifier_mvx_esdt_safe_contract_code: BytesValue,
    testing_sc_contract_code: BytesValue,
    fee_market_contract_code: BytesValue,
    chain_config_contract_code: BytesValue,
    pub state: State,
}

impl MvxEsdtSafeInteract {
    pub async fn new(config: Config) -> Self {
        let mut interactor = Interactor::new(config.gateway_uri())
            .await
            .use_chain_simulator(config.use_chain_simulator());

        interactor.set_current_dir_from_workspace("mvx-esdt-safe");
        let wallet_address = interactor.register_wallet(test_wallets::mike()).await;
        let bob_address = interactor.register_wallet(test_wallets::bob()).await;

        // Useful in the chain simulator setting
        // generate blocks until ESDTSystemSCAddress is enabled
        interactor.generate_blocks_until_epoch(1).await.unwrap();

        let mvx_esdt_safe_contract_code = BytesValue::interpret_from(
            "mxsc:../../../mvx-esdt-safe/output/mvx-esdt-safe.mxsc.json",
            &InterpreterContext::default(),
        );

        let header_verifier_mvx_esdt_safe_contract_code = BytesValue::interpret_from(
            "mxsc:../../../header-verifier/output/header-verifier.mxsc.json",
            &InterpreterContext::default(),
        );

        let fee_market_contract_code = BytesValue::interpret_from(
            "mxsc:../../../fee-market/output/fee-market.mxsc.json",
            &InterpreterContext::default(),
        );

        let testing_sc_contract_code = BytesValue::interpret_from(
            "mxsc:../../../testing-sc/output/testing-sc.mxsc.json",
            &InterpreterContext::default(),
        );

        let chain_config_contract_code = BytesValue::interpret_from(
            "mxsc:../../../chain-config/output/chain-config.mxsc.json",
            &InterpreterContext::default(),
        );

        MvxEsdtSafeInteract {
            interactor,
            wallet_address,
            bob_address,
            mvx_esdt_safe_contract_code,
            header_verifier_mvx_esdt_safe_contract_code,
            testing_sc_contract_code,
            fee_market_contract_code,
            chain_config_contract_code,
            state: State::load_state(),
        }
    }

    pub async fn load_account_state(&mut self) {
        let set_state_response = self.interactor.set_state_for_saved_accounts().await;
        self.interactor.generate_blocks(2u64).await.unwrap();
        assert!(set_state_response.is_ok());
    }

    // Arguments should be in plain text, they will be converted to hex
    pub async fn check_account_storage(
        &mut self,
        address: Address,
        wanted_key: &str,
        expected_value: Option<&str>,
    ) {
        let pairs = self.interactor.get_account_storage(&address).await;

        let wanted_hex = hex::encode(wanted_key);

        let found_entry = pairs.iter().find(|(key, _)| key.contains(&wanted_hex));

        match (found_entry, expected_value) {
            (Some(_), None) => {
                panic!(
                    "Unexpected key containing '{}' found in account storage.",
                    wanted_key
                );
            }
            (None, None) => {
                println!(
                    "Confirmed: No key containing '{}' found in account storage.",
                    wanted_key
                );
            }
            (Some((_, value)), Some(expected)) => {
                let wanted_value = hex::encode(expected);
                assert_eq!(
                    value, &wanted_value,
                    "Mismatch: expected '{}', but found '{}'",
                    expected, value
                );
            }
            (None, Some(_)) => {
                panic!(
                    "Expected key containing '{}' was not found in account storage.",
                    wanted_key
                );
            }
        }
    }

    pub async fn deploy_mvx_esdt_safe(
        &mut self,
        header_verifier_address: Bech32Address,
        opt_config: OptionalValue<EsdtSafeConfig<StaticApi>>,
    ) {
        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .init(header_verifier_address, opt_config)
            .code(&self.mvx_esdt_safe_contract_code)
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
            .from(&self.wallet_address)
            .gas(30_000_000u64)
            .typed(HeaderverifierProxy)
            .init()
            .code(&self.header_verifier_mvx_esdt_safe_contract_code)
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
            .from(&self.wallet_address)
            .gas(60_000_000u64)
            .typed(FeeMarketProxy)
            .init(esdt_safe_address, fee)
            .code(&self.fee_market_contract_code)
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
            .from(&self.wallet_address)
            .gas(30_000_000u64)
            .typed(TestingScProxy)
            .init()
            .code(&self.testing_sc_contract_code)
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
            .from(&self.wallet_address)
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
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(30_000_000u64)
            .typed(ChainConfigContractProxy)
            .init(
                config.min_validators as usize,
                config.max_validators as usize,
                config.min_stake,
                self.wallet_address.clone(),
                MultiValueEncoded::new(),
            )
            .code(self.chain_config_contract_code.clone())
            .returns(ReturnsNewAddress)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn register_operation(
        &mut self,
        signature: ManagedBuffer<StaticApi>,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        operations_hashes: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>>,
    ) {
        self.interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_header_verifier_address())
            .gas(30_000_000u64)
            .typed(HeaderverifierProxy)
            .register_bridge_operations(signature, hash_of_hashes, operations_hashes)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
    }

    pub async fn upgrade(&mut self) {
        let response = self
            .interactor
            .tx()
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .from(&self.wallet_address)
            .gas(30_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .upgrade()
            .code(&self.mvx_esdt_safe_contract_code)
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
            .from(&self.wallet_address)
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
            .from(&self.wallet_address)
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .gas(30_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .set_fee_market_address(fee_market_address)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn set_fee(&mut self, fee: FeeStruct<StaticApi>) {
        self.interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_fee_market_address())
            .gas(30_000_000u64)
            .typed(FeeMarketProxy)
            .set_fee(fee)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;
    }

    pub async fn deposit(
        &mut self,
        to: Address,
        opt_transfer_data: OptionalValueTransferDataTuple<StaticApi>,
        payments: PaymentsVec<StaticApi>,
        expected_error_message: Option<&str>,
    ) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .gas(90_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .deposit(to, opt_transfer_data)
            .payment(payments)
            .returns(ReturnsHandledOrError::new())
            .run()
            .await;

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

    pub async fn execute_operations(
        &mut self,
        hash_of_hashes: ManagedBuffer<StaticApi>,
        operation: Operation<StaticApi>,
    ) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .gas(30_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .execute_operations(hash_of_hashes, operation)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn register_token(
        &mut self,
        sov_token_id: &str,
        token_type: EsdtTokenType,
        token_display_name: &str,
        token_ticker: &str,
        num_decimals: u32,
        egld_amount: BigUint<StaticApi>,
    ) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_mvx_esdt_safe_contract_address())
            .gas(30_000_000u64)
            .typed(MvxEsdtSafeProxy)
            .register_token(
                TokenIdentifier::from(sov_token_id),
                token_type,
                token_display_name,
                token_ticker,
                num_decimals,
            )
            .egld(egld_amount)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Result: {response:?}");
    }

    pub async fn pause_endpoint(&mut self) {
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
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
            .from(&self.wallet_address)
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
            .from(&self.wallet_address)
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

    pub fn get_operation_hash(
        &mut self,
        operation: &Operation<StaticApi>,
    ) -> ManagedBuffer<StaticApi> {
        let mut serialized_operation: ManagedBuffer<StaticApi> = ManagedBuffer::new();
        let _ = operation.top_encode(&mut serialized_operation);
        let sha256 = sha256(&serialized_operation.to_vec());

        ManagedBuffer::new_from_bytes(&sha256)
    }
}
