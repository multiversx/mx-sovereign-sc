use common_test_setup::{
    constants::{
        CHAIN_CONFIG_ADDRESS, CHAIN_FACTORY_SC_ADDRESS, CROWD_TOKEN_ID, ENSHRINE_BALANCE,
        ENSHRINE_SC_ADDRESS, FEE_MARKET_ADDRESS, FUNGIBLE_TOKEN_ID, HEADER_VERIFIER_ADDRESS,
        INSUFFICIENT_WEGLD_ADDRESS, NFT_TOKEN_ID, OWNER_ADDRESS, OWNER_BALANCE,
        PREFIX_NFT_TOKEN_ID, RECEIVER_ADDRESS, SOVEREIGN_TOKEN_PREFIX, TOKEN_HANDLER_SC_ADDRESS,
        USER_ADDRESS, WEGLD_IDENTIFIER,
    },
    AccountSetup, BaseSetup,
};
use enshrine_esdt_safe::common::storage::CommonStorage;
use multiversx_sc::{
    imports::OptionalValue,
    types::{
        BigUint, EsdtTokenPayment, ManagedBuffer, ManagedVec, MultiValueEncoded, TestAddress,
        TestTokenIdentifier, TokenIdentifier,
    },
};
use multiversx_sc_scenario::{
    api::StaticApi, multiversx_chain_vm::crypto_functions::sha256, DebugApi, ReturnsHandledOrError,
    ReturnsLogs, ScenarioTxRun, ScenarioTxWhitebox,
};
use proxies::{
    enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy, header_verifier_proxy::HeaderverifierProxy,
    token_handler_proxy::TokenHandlerProxy,
};
use structs::{
    aliases::{GasLimit, OptionalValueTransferDataTuple, PaymentsVec},
    configs::{EsdtSafeConfig, SovereignConfig},
    fee::{FeeStruct, FeeType},
    operation::Operation,
};

pub struct EnshrineTestState {
    pub common_setup: BaseSetup,
}

impl EnshrineTestState {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let enshrine_esdt_owner_address = AccountSetup {
            address: OWNER_ADDRESS.to_address(),
            code_path: None,
            egld_balance: Some(OWNER_BALANCE.into()),
            esdt_balances: Some(vec![
                (CROWD_TOKEN_ID, 0, ENSHRINE_BALANCE.into()),
                (WEGLD_IDENTIFIER, 0, ENSHRINE_BALANCE.into()),
                (FUNGIBLE_TOKEN_ID, 0, ENSHRINE_BALANCE.into()),
                (NFT_TOKEN_ID, 1, ENSHRINE_BALANCE.into()),
                (PREFIX_NFT_TOKEN_ID, 1, ENSHRINE_BALANCE.into()),
            ]),
        };

        let user_address = AccountSetup {
            address: USER_ADDRESS.to_address(),
            code_path: None,
            egld_balance: Some(OWNER_BALANCE.into()),
            esdt_balances: Some(vec![
                (CROWD_TOKEN_ID, 0, ENSHRINE_BALANCE.into()),
                (NFT_TOKEN_ID, 1, ENSHRINE_BALANCE.into()),
            ]),
        };

        let insufficient_wegld_address = AccountSetup {
            address: INSUFFICIENT_WEGLD_ADDRESS.to_address(),
            code_path: None,
            egld_balance: Some(OWNER_BALANCE.into()),
            esdt_balances: Some(vec![
                (CROWD_TOKEN_ID, 0, ENSHRINE_BALANCE.into()),
                (NFT_TOKEN_ID, 1, ENSHRINE_BALANCE.into()),
                (WEGLD_IDENTIFIER, 0, ENSHRINE_BALANCE.into()),
            ]),
        };

        let receiver_address = AccountSetup {
            address: RECEIVER_ADDRESS.to_address(),
            code_path: None,
            egld_balance: None,
            esdt_balances: None,
        };

        let account_setups = vec![
            enshrine_esdt_owner_address,
            user_address,
            insufficient_wegld_address,
            receiver_address,
        ];

        let common_setup = BaseSetup::new(account_setups);

        Self { common_setup }
    }

    pub fn set_unpaused(&mut self) {
        self.common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ENSHRINE_SC_ADDRESS)
            .typed(EnshrineEsdtSafeProxy)
            .unpause_endpoint()
            .run();
    }

    pub fn set_header_verifier_address(&mut self) {
        self.common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ENSHRINE_SC_ADDRESS)
            .typed(EnshrineEsdtSafeProxy)
            .set_header_verifier_address(HEADER_VERIFIER_ADDRESS)
            .run();
    }

    pub fn setup_contracts(
        &mut self,
        is_sovereign_chain: bool,
        fee_struct: Option<&FeeStruct<StaticApi>>,
        opt_config: Option<EsdtSafeConfig<StaticApi>>,
    ) -> &mut Self {
        self.common_setup.deploy_enshrine_esdt_contract(
            is_sovereign_chain,
            Some(TokenIdentifier::from(WEGLD_IDENTIFIER)),
            Some(SOVEREIGN_TOKEN_PREFIX.into()),
            opt_config,
        );
        self.set_unpaused();
        self.common_setup
            .deploy_chain_config(SovereignConfig::default_config());
        self.common_setup
            .deploy_header_verifier(CHAIN_CONFIG_ADDRESS);
        self.register_esdt_in_header_verifier();
        self.common_setup.complete_header_verifier_setup_phase(None);
        self.common_setup.deploy_token_handler();
        self.common_setup
            .deploy_fee_market(fee_struct.cloned(), ENSHRINE_SC_ADDRESS);
        self.set_header_verifier_address();
        self.register_fee_market_address();
        self.common_setup.deploy_chain_factory();

        self
    }

    pub fn execute_operation(
        &mut self,
        error_message: Option<&str>,
        operation: Operation<StaticApi>,
        expected_log: Option<&str>,
    ) {
        let operation_hash = self.common_setup.get_operation_hash(&operation);
        let hash_of_hashes: ManagedBuffer<StaticApi> =
            ManagedBuffer::from(&sha256(&operation_hash.to_vec()));

        let (response, logs) = self
            .common_setup
            .world
            .tx()
            .from(USER_ADDRESS)
            .to(ENSHRINE_SC_ADDRESS)
            .typed(EnshrineEsdtSafeProxy)
            .execute_operations(hash_of_hashes, operation)
            .returns(ReturnsHandledOrError::new())
            .returns(ReturnsLogs)
            .run();

        self.common_setup
            .assert_expected_error_message(response, error_message);

        if let Some(expected_log) = expected_log {
            self.common_setup.assert_expected_log(logs, expected_log);
        }
    }

    pub fn register_fee_market_address(&mut self) {
        self.common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ENSHRINE_SC_ADDRESS)
            .typed(EnshrineEsdtSafeProxy)
            .set_fee_market_address(FEE_MARKET_ADDRESS)
            .run();
    }

    pub fn add_token_to_whitelist(
        &mut self,
        tokens: MultiValueEncoded<StaticApi, TokenIdentifier<StaticApi>>,
    ) {
        self.common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(ENSHRINE_SC_ADDRESS)
            .typed(EnshrineEsdtSafeProxy)
            .add_tokens_to_whitelist(tokens)
            .run();
    }

    pub fn register_tokens(
        &mut self,
        sender: &TestAddress,
        fee_payment: EsdtTokenPayment<StaticApi>,
        tokens_to_register: Vec<TestTokenIdentifier>,
        error_message: Option<&str>,
    ) {
        let mut managed_token_ids: MultiValueEncoded<StaticApi, TokenIdentifier<StaticApi>> =
            MultiValueEncoded::new();

        for token_id in tokens_to_register {
            managed_token_ids.push(TokenIdentifier::from(token_id))
        }

        let response = self
            .common_setup
            .world
            .tx()
            .from(*sender)
            .to(ENSHRINE_SC_ADDRESS)
            .typed(EnshrineEsdtSafeProxy)
            .register_new_token_id(managed_token_ids)
            .esdt(fee_payment)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(response, error_message);
    }

    pub fn deposit(
        &mut self,
        from: TestAddress,
        to: TestAddress,
        payment: PaymentsVec<StaticApi>,
        deposit_args: OptionalValueTransferDataTuple<StaticApi>,
        error_message: Option<&str>,
    ) {
        let response = self
            .common_setup
            .world
            .tx()
            .from(from)
            .to(ENSHRINE_SC_ADDRESS)
            .typed(EnshrineEsdtSafeProxy)
            .deposit(to, deposit_args)
            .payment(payment)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(response, error_message);
    }

    pub fn whitelist_enshrine_esdt(&mut self) {
        self.common_setup
            .world
            .tx()
            .from(CHAIN_FACTORY_SC_ADDRESS)
            .to(TOKEN_HANDLER_SC_ADDRESS)
            .typed(TokenHandlerProxy)
            .whitelist_enshrine_esdt(ENSHRINE_SC_ADDRESS)
            .run();
    }

    pub fn register_esdt_in_header_verifier(&mut self) {
        self.common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .to(HEADER_VERIFIER_ADDRESS)
            .typed(HeaderverifierProxy)
            .set_esdt_safe_address(ENSHRINE_SC_ADDRESS)
            .run();
    }

    pub fn setup_transfer_data(
        &mut self,
        gas_limit: GasLimit,
        function: ManagedBuffer<StaticApi>,
        args: ManagedVec<StaticApi, ManagedBuffer<StaticApi>>,
    ) -> OptionalValueTransferDataTuple<StaticApi> {
        OptionalValue::Some((gas_limit, function, MultiValueEncoded::from(args)).into())
    }

    pub fn setup_fee_struct(
        &mut self,
        base_token: TestTokenIdentifier,
        per_transfer: &BigUint<StaticApi>,
        per_gas: &BigUint<StaticApi>,
    ) -> FeeStruct<StaticApi> {
        let fee_type = FeeType::Fixed {
            token: base_token.into(),
            per_transfer: per_transfer.clone(),
            per_gas: per_gas.clone(),
        };

        FeeStruct {
            base_token: base_token.into(),
            fee_type,
        }
    }

    pub fn check_paid_issued_token_storage(&mut self, tokens: Vec<TestTokenIdentifier>) {
        self.common_setup
            .world
            .query()
            .to(ENSHRINE_SC_ADDRESS)
            .whitebox(enshrine_esdt_safe::contract_obj, |sc| {
                for token in tokens.iter() {
                    let token_id: TokenIdentifier<DebugApi> = (*token).into();
                    assert!(
                        sc.paid_issued_tokens().contains(&token_id),
                        "Token {:?} not found in storage",
                        token_id
                    );
                }
            });
    }

    pub fn check_paid_issued_token_storage_is_empty(&mut self) {
        self.common_setup
            .world
            .query()
            .to(ENSHRINE_SC_ADDRESS)
            .whitebox(enshrine_esdt_safe::contract_obj, |sc| {
                assert!(sc.paid_issued_tokens().is_empty(), "Storage is not empty");
            });
    }
}
