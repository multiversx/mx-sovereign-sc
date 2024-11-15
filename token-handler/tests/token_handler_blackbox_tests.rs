use multiversx_sc::types::{
    BigUint, EsdtLocalRole, EsdtTokenData, EsdtTokenPayment, ManagedAddress, ManagedBuffer,
    MultiValueEncoded, TestAddress, TestSCAddress, TestTokenIdentifier,
};
use multiversx_sc_scenario::{api::StaticApi, imports::MxscPath, ScenarioWorld};
use multiversx_sc_scenario::{ExpectError, ScenarioTxRun};
use proxies::chain_factory_proxy::ChainFactoryContractProxy;
use proxies::token_handler_proxy::TokenHandlerProxy;
use transaction::{OperationEsdtPayment, TransferData};

const TOKEN_HANDLER_ADDRESS: TestSCAddress = TestSCAddress::new("token-handler");
const TOKEN_HANDLER_CODE_PATH: MxscPath = MxscPath::new("output/token-handler.mxsc.json");
const OWNER_ADDRESS: TestAddress = TestAddress::new("token-handler-owner");

const USER_ADDRESS: TestAddress = TestAddress::new("user");

const FACTORY_ADDRESS: TestSCAddress = TestSCAddress::new("factorySC");
const FACTORY_CODE_PATH: MxscPath =
    MxscPath::new("../chain-factory/output/chain-factory.mxsc.json");

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
    blockchain.register_contract(FACTORY_CODE_PATH, chain_factory::ContractBuilder);

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
            .typed(TokenHandlerProxy)
            .init()
            .code(TOKEN_HANDLER_CODE_PATH)
            .new_address(TOKEN_HANDLER_ADDRESS)
            .run();

        self
    }

    fn propose_deploy_factory_sc(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(ChainFactoryContractProxy)
            .init(
                FACTORY_ADDRESS,
                FACTORY_ADDRESS,
                FACTORY_ADDRESS,
                FACTORY_ADDRESS,
            )
            .code(FACTORY_CODE_PATH)
            .new_address(FACTORY_ADDRESS)
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
                .typed(TokenHandlerProxy)
                .transfer_tokens(opt_transfer_data, to, tokens)
                .multi_esdt(payment)
                .returns(ExpectError(10, "action is not allowed"))
                .run(),
            Option::None => self
                .world
                .tx()
                .from(caller)
                .to(TOKEN_HANDLER_ADDRESS)
                .typed(TokenHandlerProxy)
                .transfer_tokens(opt_transfer_data, to, tokens)
                .returns(ExpectError(10, "action is not allowed"))
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
                .typed(TokenHandlerProxy)
                .whitelist_enshrine_esdt(enshrine_address)
                .run(),
            Some(error_status) => self
                .world
                .tx()
                .to(TOKEN_HANDLER_ADDRESS)
                .from(caller)
                .typed(TokenHandlerProxy)
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
                token_identifier: (*token_id).into(),
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
    state.propose_deploy_factory_sc();
}

#[test]
fn test_whitelist_ensrhine_esdt_caller_not_owner() {
    let mut state = TokenHandlerTestState::new();
    let error = ErrorStatus {
        code: 4,
        message: "Endpoint can only be called by owner",
    };

    state.propose_deploy_token_handler();
    state.propose_whitelist_caller(USER_ADDRESS, FACTORY_ADDRESS, Some(error));
}

#[test]
fn test_whitelist_ensrhine() {
    let mut state = TokenHandlerTestState::new();

    state.propose_deploy_token_handler();
    state.propose_whitelist_caller(OWNER_ADDRESS, FACTORY_ADDRESS, None);
}

// NOTE:
// This test at the moment is expected to fail since there is no way
// to give the correct permissions to the TokenHandler SC
#[test]
fn test_transfer_tokens_no_payment() {
    let mut state = TokenHandlerTestState::new();
    let token_ids = [NFT_TOKEN_ID, FUNGIBLE_TOKEN_ID];
    let tokens = state.setup_payments(&token_ids.to_vec());
    let esdt_payment = Option::None;
    let opt_transfer_data = Option::None;

    state.propose_deploy_token_handler();
    state.propose_deploy_factory_sc();

    state
        .world
        .set_esdt_balance(FACTORY_ADDRESS, b"NFT_TOKEN_ID", 100);
    state
        .world
        .set_esdt_balance(FACTORY_ADDRESS, b"FUNGIBLE_TOKEN_ID", 100);

    state.propose_whitelist_caller(OWNER_ADDRESS, FACTORY_ADDRESS, None);

    state.world.set_esdt_local_roles(
        TOKEN_HANDLER_ADDRESS,
        b"NFT_TOKEN_ID",
        &[
            EsdtLocalRole::NftCreate,
            EsdtLocalRole::Mint,
            EsdtLocalRole::NftBurn,
        ],
    );

    state.propose_transfer_tokens(
        FACTORY_ADDRESS,
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
