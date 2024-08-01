use multiversx_sc::types::{
    BigUint, EsdtTokenPayment, ManagedAddress, ManagedBuffer, MultiValueEncoded, TestAddress,
    TestSCAddress, TestTokenIdentifier,
};
use multiversx_sc_scenario::ScenarioTxRun;
use multiversx_sc_scenario::{api::StaticApi, imports::MxscPath, ScenarioWorld};
use token_handler::token_handler_proxy;
use transaction::{OperationEsdtPayment, StolenFromFrameworkEsdtTokenData, TransferData};

const TOKEN_HANDLER_ADDRESS: TestSCAddress = TestSCAddress::new("token-handler");
const TOKEN_HANDLER_CODE_PATH: MxscPath = MxscPath::new("output/token-handler.mxsc.json");
const OWNER_ADDRESS: TestAddress = TestAddress::new("token-handler-owner");
const USER_ADDRESS: TestAddress = TestAddress::new("user");

const NFT_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("NFT-123456");
const CROWD_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("CROWD-123456");
const FUNGIBLE_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("FUNG-123456");
const _PREFIX_NFT_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("sov-NFT-123456");

const WEGLD_BALANCE: u128 = 100_000_000_000_000_000;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(TOKEN_HANDLER_CODE_PATH, token_handler::ContractBuilder);

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

    fn propose_deploy(&mut self) -> &mut Self {
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

    fn propose_transfer_tokens(
        &mut self,
        esdt_payment: Option<EsdtTokenPayment<StaticApi>>,
        opt_transfer_data: Option<TransferData<StaticApi>>,
        to: ManagedAddress<StaticApi>,
        tokens: MultiValueEncoded<StaticApi, OperationEsdtPayment<StaticApi>>,
    ) {
        match esdt_payment {
            Option::Some(payment) => self
                .world
                .tx()
                .from(OWNER_ADDRESS)
                .to(TOKEN_HANDLER_ADDRESS)
                .typed(token_handler_proxy::TokenHandlerProxy)
                .transfer_tokens(opt_transfer_data, to, tokens)
                .multi_esdt(payment)
                .run(),
            Option::None => self
                .world
                .tx()
                .from(OWNER_ADDRESS)
                .to(TOKEN_HANDLER_ADDRESS)
                .typed(token_handler_proxy::TokenHandlerProxy)
                .transfer_tokens(opt_transfer_data, to, tokens)
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
                token_data: StolenFromFrameworkEsdtTokenData::default(),
            };

            tokens.push(payment);
        }

        tokens
    }
}

#[test]
fn test_deploy() {
    let mut state = TokenHandlerTestState::new();

    state.propose_deploy();
}

#[test]
fn test_transfer_tokens_no_payment() {
    let mut state = TokenHandlerTestState::new();
    let token_ids = [NFT_TOKEN_ID, FUNGIBLE_TOKEN_ID];
    let tokens = state.setup_payments(&token_ids.to_vec());
    let esdt_payment = Option::None;
    let opt_transfer_data = Option::None;

    state.propose_deploy();

    state.propose_transfer_tokens(
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

    state.propose_deploy();

    state.propose_transfer_tokens(
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
