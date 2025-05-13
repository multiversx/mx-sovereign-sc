use common_test_setup::{
    constants::{
        CHAIN_FACTORY_CODE_PATH, CHAIN_FACTORY_SC_ADDRESS, CROWD_TOKEN_ID, FUNGIBLE_TOKEN_ID,
        NFT_TOKEN_ID, ONE_HUNDRED_THOUSAND, OWNER_ADDRESS, OWNER_BALANCE, TOKEN_HANDLER_SC_ADDRESS,
        USER_ADDRESS,
    },
    AccountSetup, BaseSetup,
};
use multiversx_sc::types::{
    EsdtTokenData, EsdtTokenPayment, ManagedAddress, MultiValueEncoded, TestSCAddress,
    TestTokenIdentifier,
};
use multiversx_sc_scenario::{api::StaticApi, ReturnsHandledOrError, ScenarioTxRun};
use proxies::{
    chain_factory_proxy::ChainFactoryContractProxy, token_handler_proxy::TokenHandlerProxy,
};
use structs::operation::{OperationEsdtPayment, TransferData};

pub struct TokenHandlerTestState {
    pub common_setup: BaseSetup,
}

impl TokenHandlerTestState {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let owner_account = AccountSetup {
            address: OWNER_ADDRESS.to_address(),
            code_path: None,
            esdt_balances: Some(vec![
                (NFT_TOKEN_ID, 1u64, ONE_HUNDRED_THOUSAND.into()),
                (FUNGIBLE_TOKEN_ID, 0u64, ONE_HUNDRED_THOUSAND.into()),
                (CROWD_TOKEN_ID, 0u64, ONE_HUNDRED_THOUSAND.into()),
            ]),
            egld_balance: Some(OWNER_BALANCE.into()),
        };

        let user_account = AccountSetup {
            address: USER_ADDRESS.to_address(),
            code_path: None,
            esdt_balances: Some(vec![
                (NFT_TOKEN_ID, 1u64, ONE_HUNDRED_THOUSAND.into()),
                (FUNGIBLE_TOKEN_ID, 0u64, ONE_HUNDRED_THOUSAND.into()),
                (CROWD_TOKEN_ID, 0u64, ONE_HUNDRED_THOUSAND.into()),
            ]),
            egld_balance: Some(OWNER_BALANCE.into()),
        };

        let account_setups = vec![owner_account, user_account];
        let common_setup = BaseSetup::new(account_setups);

        Self { common_setup }
    }

    pub fn propose_deploy_factory_sc(&mut self) -> &mut Self {
        self.common_setup
            .world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(ChainFactoryContractProxy)
            .init(
                CHAIN_FACTORY_SC_ADDRESS,
                CHAIN_FACTORY_SC_ADDRESS,
                CHAIN_FACTORY_SC_ADDRESS,
                CHAIN_FACTORY_SC_ADDRESS,
                CHAIN_FACTORY_SC_ADDRESS,
            )
            .code(CHAIN_FACTORY_CODE_PATH)
            .new_address(CHAIN_FACTORY_SC_ADDRESS)
            .run();

        self
    }

    pub fn propose_transfer_tokens(
        &mut self,
        caller: TestSCAddress,
        esdt_payment: Option<EsdtTokenPayment<StaticApi>>,
        opt_transfer_data: Option<TransferData<StaticApi>>,
        to: ManagedAddress<StaticApi>,
        tokens: MultiValueEncoded<StaticApi, OperationEsdtPayment<StaticApi>>,
        error_message: Option<&str>,
    ) {
        let response = match esdt_payment {
            Option::Some(payment) => self
                .common_setup
                .world
                .tx()
                .from(caller)
                .to(TOKEN_HANDLER_SC_ADDRESS)
                .typed(TokenHandlerProxy)
                .transfer_tokens(opt_transfer_data, to, tokens)
                .multi_esdt(payment)
                .returns(ReturnsHandledOrError::new())
                .run(),
            Option::None => self
                .common_setup
                .world
                .tx()
                .from(caller)
                .to(TOKEN_HANDLER_SC_ADDRESS)
                .typed(TokenHandlerProxy)
                .transfer_tokens(opt_transfer_data, to, tokens)
                .returns(ReturnsHandledOrError::new())
                .run(),
        };
        self.common_setup
            .assert_expected_error_message(response, error_message);
    }

    pub fn propose_whitelist_caller(
        &mut self,
        enshrine_address: TestSCAddress,
        error_message: Option<&str>,
    ) {
        let response = self
            .common_setup
            .world
            .tx()
            .to(TOKEN_HANDLER_SC_ADDRESS)
            .from(enshrine_address)
            .typed(TokenHandlerProxy)
            .whitelist_enshrine_esdt(enshrine_address)
            .returns(ReturnsHandledOrError::new())
            .run();

        self.common_setup
            .assert_expected_error_message(response, error_message);
    }

    pub fn setup_payments(
        &mut self,
        token_ids: &Vec<TestTokenIdentifier>,
    ) -> MultiValueEncoded<StaticApi, OperationEsdtPayment<StaticApi>> {
        let mut tokens: MultiValueEncoded<StaticApi, OperationEsdtPayment<StaticApi>> =
            MultiValueEncoded::new();

        for token_id in token_ids {
            let payment: OperationEsdtPayment<StaticApi> = OperationEsdtPayment {
                token_identifier: (*token_id).into(),
                token_nonce: 1,
                token_data: EsdtTokenData::default(),
            };

            tokens.push(payment);
        }

        tokens
    }
}
