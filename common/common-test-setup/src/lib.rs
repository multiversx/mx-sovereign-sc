#![no_std]
pub mod constants;
use constants::{
    CHAIN_CONFIG_ADDRESS, CHAIN_CONFIG_CODE_PATH, CHAIN_FACTORY_CODE_PATH,
    CHAIN_FACTORY_SC_ADDRESS, DEPLOY_COST, ENSHRINE_ESDT_SAFE_CODE_PATH, ENSHRINE_SC_ADDRESS,
    ESDT_SAFE_ADDRESS, FEE_MARKET_ADDRESS, FEE_MARKET_CODE_PATH, HEADER_VERIFIER_ADDRESS,
    HEADER_VERIFIER_CODE_PATH, MVX_ESDT_SAFE_CODE_PATH, OWNER_ADDRESS, SOVEREIGN_FORGE_CODE_PATH,
    SOVEREIGN_FORGE_SC_ADDRESS, SOV_ESDT_SAFE_CODE_PATH, TESTING_SC_ADDRESS, TESTING_SC_CODE_PATH,
    TOKEN_HANDLER_CODE_PATH, TOKEN_HANDLER_SC_ADDRESS,
};
use cross_chain::storage::CrossChainStorage;
use error_messages::EMPTY_EXPECTED_LOG;
use header_verifier::{Headerverifier, OperationHashStatus};
use multiversx_sc_scenario::{
    api::StaticApi,
    imports::{
        Address, BigUint, EsdtTokenType, ManagedBuffer, MultiValue3, MultiValueEncoded, MxscPath,
        OptionalValue, ReturnsResultUnmanaged, TestSCAddress, TestTokenIdentifier, TokenIdentifier,
        TopEncode, UserBuiltinProxy, Vec,
    },
    multiversx_chain_vm::crypto_functions::sha256,
    scenario_model::{Log, TxResponseStatus},
    ReturnsHandledOrError, ScenarioTxRun, ScenarioTxWhitebox, ScenarioWorld,
};
use mvx_esdt_safe::bridging_mechanism::BridgingMechanism;
use proxies::{
    chain_config_proxy::ChainConfigContractProxy, chain_factory_proxy::ChainFactoryContractProxy,
    enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy, fee_market_proxy::FeeMarketProxy,
    header_verifier_proxy::HeaderverifierProxy, mvx_esdt_safe_proxy::MvxEsdtSafeProxy,
    sov_esdt_safe_proxy::SovEsdtSafeProxy, sovereign_forge_proxy::SovereignForgeProxy,
    testing_sc_proxy::TestingScProxy, token_handler_proxy::TokenHandlerProxy,
};
use structs::{
    configs::{EsdtSafeConfig, SovereignConfig},
    fee::FeeStruct,
    forge::{ContractInfo, ScArray},
    operation::Operation,
};

pub struct RegisterTokenArgs<'a> {
    pub sov_token_id: TokenIdentifier<StaticApi>,
    pub token_type: EsdtTokenType,
    pub token_display_name: &'a str,
    pub token_ticker: &'a str,
    pub num_decimals: usize,
}

pub struct BaseSetup {
    pub world: ScenarioWorld,
}

pub struct AccountSetup<'a> {
    pub address: Address,
    pub code_path: Option<MxscPath<'a>>,
    pub esdt_balances: Option<Vec<(TestTokenIdentifier<'a>, u64, BigUint<StaticApi>)>>,
    pub egld_balance: Option<BigUint<StaticApi>>,
}

pub enum CallerAddress {
    Owner,
    SafeSC,
}

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(FEE_MARKET_CODE_PATH, fee_market::ContractBuilder);
    blockchain.register_contract(HEADER_VERIFIER_CODE_PATH, header_verifier::ContractBuilder);
    blockchain.register_contract(CHAIN_CONFIG_CODE_PATH, chain_config::ContractBuilder);
    blockchain.register_contract(TESTING_SC_CODE_PATH, testing_sc::ContractBuilder);
    blockchain.register_contract(CHAIN_FACTORY_CODE_PATH, chain_factory::ContractBuilder);
    blockchain.register_contract(SOVEREIGN_FORGE_CODE_PATH, sovereign_forge::ContractBuilder);
    blockchain.register_contract(
        ENSHRINE_ESDT_SAFE_CODE_PATH,
        enshrine_esdt_safe::ContractBuilder,
    );
    blockchain.register_contract(TOKEN_HANDLER_CODE_PATH, token_handler::ContractBuilder);
    blockchain.register_contract(MVX_ESDT_SAFE_CODE_PATH, mvx_esdt_safe::ContractBuilder);

    blockchain
}

impl BaseSetup {
    pub fn new(account_setups: Vec<AccountSetup>) -> Self {
        let mut world = world();

        for acc in account_setups {
            let mut acc_builder = match acc.code_path {
                Some(code_path) => world.account(acc.address.clone()).code(code_path).nonce(1),
                None => world.account(acc.address.clone()).nonce(1),
            };

            if let Some(esdt_balances) = &acc.esdt_balances {
                for (token_id, nonce, amount) in esdt_balances {
                    acc_builder = if *nonce != 0 {
                        acc_builder.esdt_nft_balance(
                            *token_id,
                            *nonce,
                            amount.clone(),
                            ManagedBuffer::new(),
                        )
                    } else {
                        acc_builder.esdt_balance(*token_id, amount.clone())
                    };
                }
            }

            if let Some(balance) = &acc.egld_balance {
                acc_builder.balance(balance.clone());
            }
        }

        Self { world }
    }

    pub fn deploy_mvx_esdt_safe(
        &mut self,
        opt_config: OptionalValue<EsdtSafeConfig<StaticApi>>,
    ) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(MvxEsdtSafeProxy)
            .init(opt_config)
            .code(MVX_ESDT_SAFE_CODE_PATH)
            .new_address(ESDT_SAFE_ADDRESS)
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(MvxEsdtSafeProxy)
            .unpause_endpoint()
            .run();

        self
    }

    pub fn deploy_fee_market(
        &mut self,
        fee: Option<FeeStruct<StaticApi>>,
        esdt_safe_address: TestSCAddress,
    ) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(FeeMarketProxy)
            .init(esdt_safe_address, fee)
            .code(FEE_MARKET_CODE_PATH)
            .new_address(FEE_MARKET_ADDRESS)
            .run();

        self
    }

    pub fn deploy_testing_sc(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(TestingScProxy)
            .init()
            .code(TESTING_SC_CODE_PATH)
            .new_address(TESTING_SC_ADDRESS)
            .run();

        self
    }

    pub fn deploy_header_verifier(&mut self, sovereign_contracts: Vec<ScArray>) -> &mut Self {
        let contracts_array = self.get_contract_info_struct_for_sc_type(sovereign_contracts);

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(HeaderverifierProxy)
            .init(MultiValueEncoded::from_iter(contracts_array))
            .code(HEADER_VERIFIER_CODE_PATH)
            .new_address(HEADER_VERIFIER_ADDRESS)
            .run();

        self
    }

    pub fn complete_header_verifier_setup_phase(&mut self, expected_error_message: Option<&str>) {
        let response = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(HEADER_VERIFIER_ADDRESS)
            .typed(HeaderverifierProxy)
            .complete_setup_phase()
            .returns(ReturnsHandledOrError::new())
            .run();

        self.assert_expected_error_message(response, expected_error_message);
    }

    pub fn complete_fee_market_setup_phase(&mut self, expected_error_message: Option<&str>) {
        let response = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(FEE_MARKET_ADDRESS)
            .typed(FeeMarketProxy)
            .complete_setup_phase()
            .returns(ReturnsHandledOrError::new())
            .run();

        self.change_ownership_to_header_verifier(FEE_MARKET_ADDRESS);

        self.assert_expected_error_message(response, expected_error_message);
    }

    pub fn complete_sovereign_forge_setup_phase(&mut self, expected_error_message: Option<&str>) {
        let response = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(SOVEREIGN_FORGE_SC_ADDRESS)
            .typed(SovereignForgeProxy)
            .complete_setup_phase()
            .returns(ReturnsHandledOrError::new())
            .run();

        self.assert_expected_error_message(response, expected_error_message);
    }

    pub fn deploy_chain_config(
        &mut self,
        opt_config: OptionalValue<SovereignConfig<StaticApi>>,
        expected_error_message: Option<&str>,
    ) -> &mut Self {
        let response = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(ChainConfigContractProxy)
            .init(opt_config)
            .code(CHAIN_CONFIG_CODE_PATH)
            .new_address(CHAIN_CONFIG_ADDRESS)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.assert_expected_error_message(response, expected_error_message);

        self
    }

    pub fn complete_chain_config_setup_phase(&mut self, expect_error: Option<&str>) {
        let transaction = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(CHAIN_CONFIG_ADDRESS)
            .typed(ChainConfigContractProxy)
            .complete_setup_phase()
            .returns(ReturnsHandledOrError::new())
            .run();

        self.assert_expected_error_message(transaction, expect_error);
    }

    pub fn deploy_chain_factory(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(ChainFactoryContractProxy)
            .init(
                SOVEREIGN_FORGE_SC_ADDRESS,
                CHAIN_CONFIG_ADDRESS,
                HEADER_VERIFIER_ADDRESS,
                ESDT_SAFE_ADDRESS,
                FEE_MARKET_ADDRESS,
            )
            .code(CHAIN_FACTORY_CODE_PATH)
            .new_address(CHAIN_FACTORY_SC_ADDRESS)
            .run();

        self
    }

    pub fn deploy_sovereign_forge(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(SovereignForgeProxy)
            .init(DEPLOY_COST)
            .code(SOVEREIGN_FORGE_CODE_PATH)
            .new_address(SOVEREIGN_FORGE_SC_ADDRESS)
            .run();

        self
    }

    pub fn deploy_enshrine_esdt_contract(
        &mut self,
        is_sovereign_chain: bool,
        wegld_identifier: Option<TokenIdentifier<StaticApi>>,
        sovereign_token_prefix: Option<ManagedBuffer<StaticApi>>,
        opt_config: Option<EsdtSafeConfig<StaticApi>>,
    ) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(EnshrineEsdtSafeProxy)
            .init(
                is_sovereign_chain,
                TOKEN_HANDLER_SC_ADDRESS,
                wegld_identifier,
                sovereign_token_prefix,
                opt_config,
            )
            .code(ENSHRINE_ESDT_SAFE_CODE_PATH)
            .new_address(ENSHRINE_SC_ADDRESS)
            .run();

        self
    }

    pub fn deploy_token_handler(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(TokenHandlerProxy)
            .init(CHAIN_FACTORY_SC_ADDRESS)
            .code(TOKEN_HANDLER_CODE_PATH)
            .new_address(TOKEN_HANDLER_SC_ADDRESS)
            .run();

        self
    }

    pub fn deploy_sov_esdt_safe(
        &mut self,
        fee_market_address: TestSCAddress,
        opt_config: OptionalValue<EsdtSafeConfig<StaticApi>>,
    ) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(SovEsdtSafeProxy)
            .init(fee_market_address, opt_config)
            .code(SOV_ESDT_SAFE_CODE_PATH)
            .new_address(ESDT_SAFE_ADDRESS)
            .run();

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .typed(SovEsdtSafeProxy)
            .unpause_endpoint()
            .run();

        self
    }

    pub fn deploy_phase_one(
        &mut self,
        payment: &BigUint<StaticApi>,
        opt_preferred_chain: Option<ManagedBuffer<StaticApi>>,
        opt_config: OptionalValue<SovereignConfig<StaticApi>>,
        error_message: Option<&str>,
    ) {
        let response = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(SOVEREIGN_FORGE_SC_ADDRESS)
            .typed(SovereignForgeProxy)
            .deploy_phase_one(opt_preferred_chain, opt_config)
            .egld(payment)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.assert_expected_error_message(response, error_message);
    }

    pub fn deploy_phase_two(
        &mut self,
        opt_config: OptionalValue<EsdtSafeConfig<StaticApi>>,
        error_message: Option<&str>,
    ) {
        let response = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(SOVEREIGN_FORGE_SC_ADDRESS)
            .typed(SovereignForgeProxy)
            .deploy_phase_two(opt_config)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.assert_expected_error_message(response, error_message);
    }

    pub fn deploy_phase_three(
        &mut self,
        fee: Option<FeeStruct<StaticApi>>,
        error_message: Option<&str>,
    ) {
        let response = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(SOVEREIGN_FORGE_SC_ADDRESS)
            .typed(SovereignForgeProxy)
            .deploy_phase_three(fee)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.assert_expected_error_message(response, error_message);
    }

    pub fn deploy_phase_four(&mut self, error_message: Option<&str>) {
        let response = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(SOVEREIGN_FORGE_SC_ADDRESS)
            .typed(SovereignForgeProxy)
            .deploy_phase_four()
            .returns(ReturnsHandledOrError::new())
            .run();

        self.assert_expected_error_message(response, error_message);
    }

    pub fn register_operation(
        &mut self,
        caller: CallerAddress,
        signature: ManagedBuffer<StaticApi>,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        operations_hashes: MultiValueEncoded<StaticApi, ManagedBuffer<StaticApi>>,
    ) {
        let from_address: Address = match caller {
            CallerAddress::SafeSC => ENSHRINE_SC_ADDRESS.to_address(),
            CallerAddress::Owner => OWNER_ADDRESS.to_address(),
        };

        self.world
            .tx()
            .from(from_address)
            .to(HEADER_VERIFIER_ADDRESS)
            .typed(HeaderverifierProxy)
            .register_bridge_operations(
                signature,
                hash_of_hashes,
                ManagedBuffer::new(),
                ManagedBuffer::new(),
                operations_hashes,
            )
            .run();
    }

    pub fn set_fee_during_setup_phase(
        &mut self,
        fee_struct: FeeStruct<StaticApi>,
        error_message: Option<&str>,
    ) {
        let response = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(FEE_MARKET_ADDRESS)
            .typed(FeeMarketProxy)
            .set_fee_during_setup_phase(fee_struct)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.assert_expected_error_message(response, error_message);
    }

    pub fn set_fee(
        &mut self,
        hash_of_hashes: &ManagedBuffer<StaticApi>,
        fee_struct: Option<FeeStruct<StaticApi>>,
        error_message: Option<&str>,
    ) {
        let response = self
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(FEE_MARKET_ADDRESS)
            .typed(FeeMarketProxy)
            .set_fee(hash_of_hashes, fee_struct.unwrap())
            .returns(ReturnsHandledOrError::new())
            .run();

        self.assert_expected_error_message(response, error_message);
    }

    pub fn change_ownership_to_header_verifier(&mut self, sc_address: TestSCAddress) {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(sc_address)
            .typed(UserBuiltinProxy)
            .change_owner_address(&HEADER_VERIFIER_ADDRESS.to_managed_address())
            .returns(ReturnsResultUnmanaged)
            .run();
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

    pub fn get_contract_info_struct_for_sc_type(
        &mut self,
        sc_array: Vec<ScArray>,
    ) -> Vec<ContractInfo<StaticApi>> {
        sc_array
            .iter()
            .map(|sc| {
                ContractInfo::new(
                    sc.clone(),
                    self.get_sc_address(sc.clone()).to_managed_address(),
                )
            })
            .collect()
    }

    pub fn get_sc_address(&mut self, sc_type: ScArray) -> TestSCAddress {
        match sc_type {
            ScArray::ChainConfig => CHAIN_CONFIG_ADDRESS,
            ScArray::ChainFactory => CHAIN_FACTORY_SC_ADDRESS,
            ScArray::ESDTSafe => ESDT_SAFE_ADDRESS,
            ScArray::HeaderVerifier => HEADER_VERIFIER_ADDRESS,
            ScArray::FeeMarket => FEE_MARKET_ADDRESS,
            ScArray::EnshrineESDTSafe => ENSHRINE_SC_ADDRESS,
            _ => TestSCAddress::new("ERROR"),
        }
    }

    pub fn check_account_multiple_esdts(
        &mut self,
        address: Address,
        tokens: Vec<MultiValue3<TestTokenIdentifier, u64, BigUint<StaticApi>>>,
    ) {
        for token in tokens {
            let (token_id, nonce, amount) = token.into_tuple();
            self.world
                .check_account(&address)
                .esdt_nft_balance_and_attributes(
                    token_id,
                    nonce,
                    amount,
                    ManagedBuffer::<StaticApi>::new(),
                );
        }
    }

    pub fn check_account_single_esdt(
        &mut self,
        address: Address,
        token_id: TestTokenIdentifier,
        nonce: u64,
        expected_balance: BigUint<StaticApi>,
    ) {
        self.world
            .check_account(address)
            .esdt_nft_balance_and_attributes(
                token_id,
                nonce,
                expected_balance,
                ManagedBuffer::<StaticApi>::new(),
            );
    }

    pub fn check_deposited_tokens_amount(&mut self, tokens: Vec<(TestTokenIdentifier, u64)>) {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ESDT_SAFE_ADDRESS)
            .whitebox(mvx_esdt_safe::contract_obj, |sc| {
                for token in tokens {
                    let (token_id, amount) = token;
                    assert!(sc.deposited_tokens_amount(&token_id.into()).get() == amount);
                }
            });
    }

    pub fn check_multiversx_to_sovereign_token_id_mapper_is_empty(&mut self, token_name: &str) {
        self.world
            .query()
            .to(ESDT_SAFE_ADDRESS)
            .whitebox(mvx_esdt_safe::contract_obj, |sc| {
                assert!(sc
                    .multiversx_to_sovereign_token_id_mapper(
                        &TestTokenIdentifier::new(token_name).into()
                    )
                    .is_empty());
            });
    }

    pub fn check_operation_hash_status_is_empty(
        &mut self,
        operation_hash: &ManagedBuffer<StaticApi>,
    ) {
        self.world.query().to(HEADER_VERIFIER_ADDRESS).whitebox(
            header_verifier::contract_obj,
            |sc| {
                let operation_hash_whitebox =
                    ManagedBuffer::new_from_bytes(&operation_hash.to_vec());
                let hash_of_hashes =
                    ManagedBuffer::new_from_bytes(&sha256(&operation_hash_whitebox.to_vec()));

                assert!(sc
                    .operation_hash_status(&hash_of_hashes, &operation_hash_whitebox)
                    .is_empty());
            },
        )
    }

    pub fn check_operation_hash_status(
        &mut self,
        operation_hash: &ManagedBuffer<StaticApi>,
        status: OperationHashStatus,
    ) {
        self.world.query().to(HEADER_VERIFIER_ADDRESS).whitebox(
            header_verifier::contract_obj,
            |sc| {
                let operation_hash_whitebox =
                    ManagedBuffer::new_from_bytes(&operation_hash.to_vec());
                let hash_of_hashes =
                    ManagedBuffer::new_from_bytes(&sha256(&operation_hash_whitebox.to_vec()));

                assert!(
                    sc.operation_hash_status(&hash_of_hashes, &operation_hash_whitebox)
                        .get()
                        == status
                );
            },
        )
    }

    //NOTE: transferValue returns an empty log and calling this function on it will panic
    pub fn assert_expected_log(&mut self, logs: Vec<Log>, expected_log: Option<&str>) {
        match expected_log {
            None => {
                assert!(
                    logs.is_empty(),
                    "Expected no logs, but found some: {:?}",
                    logs
                );
            }
            Some(expected_str) => {
                assert!(!expected_str.is_empty(), "{}", EMPTY_EXPECTED_LOG);
                let expected_bytes = ManagedBuffer::<StaticApi>::from(expected_str).to_vec();

                let found_log = logs
                    .iter()
                    .find(|log| log.topics.iter().any(|topic| *topic == expected_bytes));

                assert!(
                    found_log.is_some(),
                    "Expected log '{}' not found",
                    expected_str
                );
            }
        }
    }

    pub fn assert_expected_data(&self, logs: Vec<Log>, expected_data: &str) {
        let expected_bytes = ManagedBuffer::<StaticApi>::from(expected_data).to_vec();

        let found = logs.iter().any(|log| {
            log.data
                .iter()
                .any(|data_item| data_item.to_vec() == expected_bytes)
        });

        assert!(found, "Expected data '{}' not found", expected_data);
    }
}
