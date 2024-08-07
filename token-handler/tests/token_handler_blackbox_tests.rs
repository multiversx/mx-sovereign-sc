use multiversx_sc::types::{
    BigUint, EsdtTokenData, EsdtTokenPayment, ManagedAddress, ManagedBuffer, MultiValueEncoded,
    TestAddress, TestSCAddress, TestTokenIdentifier,
};
use multiversx_sc_scenario::{api::StaticApi, imports::MxscPath, ScenarioWorld};
use multiversx_sc_scenario::{ExpectError, ScenarioTxRun};
use token_handler::{dummy_enshrine_proxy, token_handler_proxy};
use transaction::{OperationEsdtPayment, TransferData};

const TOKEN_HANDLER_ADDRESS: TestSCAddress = TestSCAddress::new("token-handler");
const TOKEN_HANDLER_CODE_PATH: MxscPath = MxscPath::new("output/token-handler.mxsc.json");
const OWNER_ADDRESS: TestAddress = TestAddress::new("token-handler-owner");

const USER_ADDRESS: TestAddress = TestAddress::new("user");

const DUMMY_ENSRHINE_ADDRESS: TestSCAddress = TestSCAddress::new("enshrine");
const DUMMY_ENSHRINE_CODE_PATH: MxscPath = MxscPath::new("../pair-mock/output/pair-mock.mxsc.json");

const NFT_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("NFT-123456");
const CROWD_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("CROWD-123456");
const FUNGIBLE_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("FUNG-123456");
const _PREFIX_NFT_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("sov-NFT-123456");

const WEGLD_BALANCE: u128 = 100_000_000_000_000_000;

pub struct ErrorStatus<'a> {
    code: u64,
    message: &'a str,
}

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(TOKEN_HANDLER_CODE_PATH, token_handler::ContractBuilder);
    blockchain.register_contract(DUMMY_ENSHRINE_CODE_PATH, pair_mock::ContractBuilder);

    blockchain
}

struct TokenHandlerTestState {
    world: ScenarioWorld,
}

impl TokenHandlerTestState {
    fn new() -> Self {
        let mut world = world();

        world
            .account(OWNER_ADDRESS)
            .esdt_nft_balance(NFT_TOKEN_ID, 1, 100_000, ManagedBuffer::new())
            .esdt_nft_balance(FUNGIBLE_TOKEN_ID, 0, 100_000, ManagedBuffer::new())
            .esdt_balance(CROWD_TOKEN_ID, 100_000)
            .balance(BigUint::from(WEGLD_BALANCE))
            .nonce(1);

        world
            .account(USER_ADDRESS)
            .esdt_nft_balance(NFT_TOKEN_ID, 1, 100_000, ManagedBuffer::new())
            .esdt_nft_balance(FUNGIBLE_TOKEN_ID, 0, 100_000, ManagedBuffer::new())
            .esdt_balance(CROWD_TOKEN_ID, 100_000)
            .balance(BigUint::from(WEGLD_BALANCE))
            .nonce(1);

        Self { world }
    }

    fn propose_deploy_token_handler(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(token_handler_proxy::TokenHandlerProxy)
            .init()
            .code(TOKEN_HANDLER_CODE_PATH)
            .new_address(TOKEN_HANDLER_ADDRESS)
            .run();

        self
    }

    fn propose_deploy_dummy_enshrine(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(dummy_enshrine_proxy::PairMockProxy)
            .init(NFT_TOKEN_ID)
            .code(DUMMY_ENSHRINE_CODE_PATH)
            .new_address(DUMMY_ENSRHINE_ADDRESS)
            .run();

        self
    }

    fn propose_transfer_tokens(
        &mut self,
        caller: TestSCAddress,
        esdt_payment: Option<EsdtTokenPayment<StaticApi>>,
        opt_transfer_data: Option<TransferData<StaticApi>>,
        to: ManagedAddress<StaticApi>,
        tokens: MultiValueEncoded<StaticApi, OperationEsdtPayment<StaticApi>>,
    ) {
        match esdt_payment {
            Option::Some(payment) => self
                .world
                .tx()
                .from(caller)
                .to(TOKEN_HANDLER_ADDRESS)
                .typed(token_handler_proxy::TokenHandlerProxy)
                .transfer_tokens(opt_transfer_data, to, tokens)
                .multi_esdt(payment)
                .run(),
            Option::None => self
                .world
                .tx()
                .from(caller)
                .to(TOKEN_HANDLER_ADDRESS)
                .typed(token_handler_proxy::TokenHandlerProxy)
                .transfer_tokens(opt_transfer_data, to, tokens)
                .run(),
        }
    }

    fn propose_whitelist_caller(
        &mut self,
        caller: TestAddress,
        enshrine_address: TestSCAddress,
        error: Option<ErrorStatus>,
    ) {
        match error {
            None => self
                .world
                .tx()
                .to(TOKEN_HANDLER_ADDRESS)
                .from(caller)
                .typed(token_handler_proxy::TokenHandlerProxy)
                .whitelist_enshrine_esdt(enshrine_address)
                .run(),
            Some(error_status) => self
                .world
                .tx()
                .to(TOKEN_HANDLER_ADDRESS)
                .from(caller)
                .typed(token_handler_proxy::TokenHandlerProxy)
                .whitelist_enshrine_esdt(enshrine_address)
                .returns(ExpectError(error_status.code, error_status.message))
                .run(),
        }
    }

    fn setup_payments(
        &mut self,
        token_ids: &Vec<TestTokenIdentifier>,
    ) -> MultiValueEncoded<StaticApi, OperationEsdtPayment<StaticApi>> {
        let mut tokens: MultiValueEncoded<StaticApi, OperationEsdtPayment<StaticApi>> =
            MultiValueEncoded::new();

        for token_id in token_ids {
            let payment: OperationEsdtPayment<StaticApi> = OperationEsdtPayment {
                token_identifier: token_id.clone().into(),
                token_nonce: 1,
                token_data: EsdtTokenData::default(),
            };

            tokens.push(payment);
        }

        tokens
    }
}

#[test]
fn test_deploy() {
    let mut state = TokenHandlerTestState::new();

    state.propose_deploy_token_handler();
    state.propose_deploy_dummy_enshrine();
}

#[test]
fn test_whitelist_ensrhine_esdt_caller_not_owner() {
    let mut state = TokenHandlerTestState::new();
    let error = ErrorStatus {
        code: 4,
        message: "Endpoint can only be called by owner",
    };

    state.propose_deploy_token_handler();
    state.propose_whitelist_caller(USER_ADDRESS, DUMMY_ENSRHINE_ADDRESS, Some(error));
}

#[test]
fn test_whitelist_ensrhine() {
    let mut state = TokenHandlerTestState::new();

    state.propose_deploy_token_handler();
    state.propose_whitelist_caller(OWNER_ADDRESS, DUMMY_ENSRHINE_ADDRESS, None);
}

#[test]
fn test_transfer_tokens_no_payment() {
    let mut state = TokenHandlerTestState::new();
    let token_ids = [NFT_TOKEN_ID, FUNGIBLE_TOKEN_ID];
    let tokens = state.setup_payments(&token_ids.to_vec());
    let esdt_payment = Option::None;
    let opt_transfer_data = Option::None;

    state.propose_deploy_token_handler();
    state.propose_deploy_dummy_enshrine();

    state
        .world
        .set_esdt_balance(DUMMY_ENSRHINE_ADDRESS, b"NFT_TOKEN_ID", 100);
    state
        .world
        .set_esdt_balance(DUMMY_ENSRHINE_ADDRESS, b"FUNGIBLE_TOKEN_ID", 100);

    state.propose_whitelist_caller(OWNER_ADDRESS, DUMMY_ENSRHINE_ADDRESS, None);

    state.propose_transfer_tokens(
        DUMMY_ENSRHINE_ADDRESS,
        esdt_payment,
        opt_transfer_data,
        USER_ADDRESS.to_managed_address(),
        tokens,
    );

    state
        .world
        .check_account(TOKEN_HANDLER_ADDRESS)
        .esdt_balance(FUNGIBLE_TOKEN_ID, 0);
}

#[test]
fn test_transfer_tokens_fungible_payment() {
    let mut state = TokenHandlerTestState::new();
    let token_ids = [NFT_TOKEN_ID, FUNGIBLE_TOKEN_ID];
    let tokens = state.setup_payments(&token_ids.to_vec());
    let esdt_payment = Option::Some(EsdtTokenPayment {
        token_identifier: FUNGIBLE_TOKEN_ID.into(),
        token_nonce: 0,
        amount: BigUint::from(100u64),
    });
    let opt_transfer_data = Option::None;

    state.propose_deploy_token_handler();
    state.propose_deploy_dummy_enshrine();

    state
        .world
        .set_esdt_balance(DUMMY_ENSRHINE_ADDRESS, b"NFT_TOKEN_ID", 100);
    state
        .world
        .set_esdt_balance(DUMMY_ENSRHINE_ADDRESS, b"FUNGIBLE_TOKEN_ID", 200);

    state.propose_transfer_tokens(
        DUMMY_ENSRHINE_ADDRESS,
        esdt_payment,
        opt_transfer_data,
        USER_ADDRESS.to_managed_address(),
        tokens,
    );

    state
        .world
        .check_account(TOKEN_HANDLER_ADDRESS)
        .esdt_balance(FUNGIBLE_TOKEN_ID, 100);
}
