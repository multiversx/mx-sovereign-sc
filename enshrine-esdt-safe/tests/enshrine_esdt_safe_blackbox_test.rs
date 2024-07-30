use bls_signature::BlsSignature;
use enshrine_esdt_safe::enshrine_esdt_safe_proxy;
use header_verifier::header_verifier_proxy;
use multiversx_sc::api::StaticVarApi;
use multiversx_sc::codec::TopEncode;
use multiversx_sc::types::{
    Address, BigUint, EsdtTokenPayment, ManagedBuffer, ManagedByteArray, ManagedVec,
    MultiValueEncoded, TestAddress, TestSCAddress, TestTokenIdentifier, TokenIdentifier,
};
use multiversx_sc_scenario::api::StaticApi;
use multiversx_sc_scenario::multiversx_chain_vm::crypto_functions::sha256;
use multiversx_sc_scenario::{imports::MxscPath, ScenarioWorld};
use multiversx_sc_scenario::{managed_address, ExpectError, ScenarioTxRun};
use token_handler::token_handler_proxy;
use transaction::{
    Operation, OperationData, OperationEsdtPayment, StolenFromFrameworkEsdtTokenData,
};

const ENSHRINE_ESDT_ADDRESS: TestSCAddress = TestSCAddress::new("enshrine-esdt");
const ENSHRINE_ESDT_CODE_PATH: MxscPath = MxscPath::new("output/enshrine-esdt-safe.mxsc-json");
const ENSHRINE_ESDT_OWNER_ADDRESS: TestAddress = TestAddress::new("enshrine-esdt-owner");

const ENSHRINE_OWNER_BALANCE: u64 = 100_000_000;
const USER_EGLD_BALANCE: u64 = 100_000_000;
const DEFAULT_ISSUE_COST: u64 = 50_000_000_000_000_000;

const HEADER_VERIFIER_ADDRESS: TestSCAddress = TestSCAddress::new("header_verifier");
const HEADER_VERIFIER_CODE_PATH: MxscPath =
    MxscPath::new("../header-verifier/output/header-verifier.mxsc.json");

const TOKEN_HANDLER_ADDRESS: TestSCAddress = TestSCAddress::new("token_handler");
const TOKEN_HANDLER_CODE_PATH: MxscPath =
    MxscPath::new("../token-handler/output/token-handler.mxsc.json");

const USER_ADDRESS: TestAddress = TestAddress::new("user");
const INSUFFICIENT_WEGLD_ADDRESS: TestAddress = TestAddress::new("insufficient_wegld");
const RECEIVER_ADDRESS: TestAddress = TestAddress::new("receiver");

const NFT_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("NFT-123456");
const CROWD_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("CROWD-123456");
const FUNGIBLE_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("FUNG-123456");
const PREFIX_NFT_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("sov-NFT-123456");

const WEGLD_IDENTIFIER: TestTokenIdentifier = TestTokenIdentifier::new("WEGLD-123456");
const WEGLD_BALANCE: u128 = 100_000_000_000_000_000;
const SOVEREIGN_TOKEN_PREFIX: &str = "sov";

pub struct ErrorStatus<'a> {
    code: u64,
    error_message: &'a str,
}

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(ENSHRINE_ESDT_CODE_PATH, enshrine_esdt_safe::ContractBuilder);
    blockchain.register_contract(HEADER_VERIFIER_CODE_PATH, header_verifier::ContractBuilder);
    blockchain.register_contract(TOKEN_HANDLER_CODE_PATH, token_handler::ContractBuilder);

    blockchain
}

struct EnshrineTestState {
    world: ScenarioWorld,
}

impl EnshrineTestState {
    fn new() -> Self {
        let mut world = world();

        world
            .account(ENSHRINE_ESDT_OWNER_ADDRESS)
            .esdt_balance(CROWD_TOKEN_ID, BigUint::from(WEGLD_BALANCE))
            .esdt_balance(WEGLD_IDENTIFIER, BigUint::from(WEGLD_BALANCE))
            .esdt_balance(FUNGIBLE_TOKEN_ID, BigUint::from(WEGLD_BALANCE))
            .esdt_nft_balance(NFT_TOKEN_ID, 1, 100_000, ManagedBuffer::new())
            .esdt_nft_balance(PREFIX_NFT_TOKEN_ID, 1, 100_000, ManagedBuffer::new())
            .nonce(1)
            .balance(ENSHRINE_OWNER_BALANCE);

        world
            .account(USER_ADDRESS)
            .esdt_nft_balance(NFT_TOKEN_ID, 1, 100_000, ManagedBuffer::new())
            .esdt_balance(CROWD_TOKEN_ID, 100_000)
            .balance(USER_EGLD_BALANCE)
            .nonce(1);

        world
            .account(INSUFFICIENT_WEGLD_ADDRESS)
            .esdt_nft_balance(NFT_TOKEN_ID, 1, 100_000, ManagedBuffer::new())
            .esdt_balance(WEGLD_IDENTIFIER, BigUint::from(WEGLD_BALANCE + 100_000))
            .esdt_balance(FUNGIBLE_TOKEN_ID, BigUint::from(WEGLD_BALANCE))
            .esdt_balance(CROWD_TOKEN_ID, BigUint::from(WEGLD_BALANCE))
            .balance(USER_EGLD_BALANCE)
            .nonce(1);

        world.account(RECEIVER_ADDRESS).nonce(1);

        Self { world }
    }

    fn propose_set_unpaused(&mut self) {
        self.world
            .tx()
            .from(ENSHRINE_ESDT_OWNER_ADDRESS)
            .to(ENSHRINE_ESDT_ADDRESS)
            .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
            .unpause_endpoint()
            .run();
    }

    fn propose_set_header_verifier_address(&mut self) {
        self.world
            .tx()
            .from(ENSHRINE_ESDT_OWNER_ADDRESS)
            .to(ENSHRINE_ESDT_ADDRESS)
            .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
            .set_header_verifier_address(HEADER_VERIFIER_ADDRESS)
            .run();
    }

    fn deploy_enshrine_esdt_contract(
        &mut self,
        is_sovereign_chain: bool,
        wegld_identifier: Option<TokenIdentifier<StaticApi>>,
        sovereign_token_prefix: Option<ManagedBuffer<StaticApi>>,
    ) -> &mut Self {
        self.world
            .tx()
            .from(ENSHRINE_ESDT_OWNER_ADDRESS)
            .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
            .init(
                is_sovereign_chain,
                TOKEN_HANDLER_ADDRESS,
                wegld_identifier,
                sovereign_token_prefix,
            )
            .code(ENSHRINE_ESDT_CODE_PATH)
            .new_address(ENSHRINE_ESDT_ADDRESS)
            .run();

        self.propose_set_unpaused();

        self
    }

    fn deploy_header_verifier_contract(&mut self) -> &mut Self {
        let bls_pub_key: ManagedBuffer<StaticApi> = ManagedBuffer::new();
        let mut bls_pub_keys = MultiValueEncoded::new();
        bls_pub_keys.push(bls_pub_key);

        self.world
            .tx()
            .from(ENSHRINE_ESDT_OWNER_ADDRESS)
            .typed(header_verifier_proxy::HeaderverifierProxy)
            .init(bls_pub_keys)
            .code(HEADER_VERIFIER_CODE_PATH)
            .new_address(HEADER_VERIFIER_ADDRESS)
            .run();

        self
    }

    fn deploy_token_handler_contract(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(ENSHRINE_ESDT_OWNER_ADDRESS)
            .typed(token_handler_proxy::TokenHandlerProxy)
            .init(SOVEREIGN_TOKEN_PREFIX)
            .code(TOKEN_HANDLER_CODE_PATH)
            .new_address(TOKEN_HANDLER_ADDRESS)
            .run();

        self
    }

    fn propose_setup_contracts(&mut self, is_sovereign_chain: bool) -> &mut Self {
        self.deploy_enshrine_esdt_contract(
            is_sovereign_chain,
            Some(TokenIdentifier::from(WEGLD_IDENTIFIER)),
            Some(SOVEREIGN_TOKEN_PREFIX.into()),
        );
        self.deploy_header_verifier_contract();
        self.deploy_token_handler_contract();
        self.propose_set_header_verifier_address();

        self
    }

    fn propose_execute_operation(
        &mut self,
        error_status: Option<ErrorStatus>,
        tokens: &Vec<TestTokenIdentifier>,
    ) {
        let (tokens, data) = self.setup_payments(tokens);
        let to = managed_address!(&Address::from(&RECEIVER_ADDRESS.eval_to_array()));
        let operation = Operation::new(to, tokens, data);
        let operation_hash = self.get_operation_hash(&operation);
        let hash_of_hashes: ManagedBuffer<StaticApi> =
            ManagedBuffer::from(&sha256(&operation_hash.to_vec()));

        match error_status {
            Some(status) => {
                self.world
                    .tx()
                    .from(USER_ADDRESS)
                    .to(ENSHRINE_ESDT_ADDRESS)
                    .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
                    .execute_operations(hash_of_hashes, operation)
                    .returns(ExpectError(status.code, status.error_message))
                    .run();
            }

            None => {
                self.world
                    .tx()
                    .from(USER_ADDRESS)
                    .to(ENSHRINE_ESDT_ADDRESS)
                    .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
                    .execute_operations(hash_of_hashes, operation)
                    .run();
            }
        }
    }

    fn propose_register_operation(&mut self, tokens: &Vec<TestTokenIdentifier>) {
        let (tokens, data) = self.setup_payments(tokens);
        let to = managed_address!(&Address::from(RECEIVER_ADDRESS.eval_to_array()));
        let operation = Operation::new(to, tokens, data);
        let operation_hash = self.get_operation_hash(&operation);
        let mut operations_hashes = MultiValueEncoded::new();

        operations_hashes.push(operation_hash.clone());

        let mock_signature = self.mock_bls_signature(&operation_hash);
        let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

        self.world
            .tx()
            .from(ENSHRINE_ESDT_OWNER_ADDRESS)
            .to(HEADER_VERIFIER_ADDRESS)
            .typed(header_verifier_proxy::HeaderverifierProxy)
            .register_bridge_operations(
                mock_signature,
                hash_of_hashes.clone(),
                operations_hashes.clone(),
            )
            .run();
    }

    fn propose_register_tokens(
        &mut self,
        sender: &TestAddress,
        fee_payment: EsdtTokenPayment<StaticApi>,
        tokens_to_register: Vec<TestTokenIdentifier>,
        error_status: Option<ErrorStatus>,
    ) {
        let mut managed_token_ids: MultiValueEncoded<StaticApi, TokenIdentifier<StaticApi>> =
            MultiValueEncoded::new();

        for token_id in tokens_to_register {
            managed_token_ids.push(TokenIdentifier::from(token_id))
        }

        match error_status {
            Some(status) => self
                .world
                .tx()
                .from(sender.clone())
                .to(ENSHRINE_ESDT_ADDRESS)
                .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
                .register_new_token_id(managed_token_ids)
                .returns(ExpectError(status.code, status.error_message))
                .esdt(fee_payment)
                .run(),
            None => self
                .world
                .tx()
                .from(sender.clone())
                .to(ENSHRINE_ESDT_ADDRESS)
                .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
                .register_new_token_id(managed_token_ids)
                .esdt(fee_payment)
                .run(),
        }
    }

    fn mock_bls_signature(
        &mut self,
        operation_hash: &ManagedBuffer<StaticApi>,
    ) -> BlsSignature<StaticApi> {
        let byte_arr: &mut [u8; 48] = &mut [0; 48];
        operation_hash.load_to_byte_array(byte_arr);
        let mock_signature: BlsSignature<StaticApi> = ManagedByteArray::new_from_bytes(byte_arr);

        mock_signature
    }

    fn setup_payments(
        &mut self,
        token_ids: &Vec<TestTokenIdentifier>,
    ) -> (
        ManagedVec<StaticApi, OperationEsdtPayment<StaticApi>>,
        OperationData<StaticApi>,
    ) {
        let mut tokens: ManagedVec<StaticApi, OperationEsdtPayment<StaticApi>> = ManagedVec::new();

        for token_id in token_ids {
            let payment: OperationEsdtPayment<StaticApi> = OperationEsdtPayment::new(
                token_id.clone().into(),
                1,
                StolenFromFrameworkEsdtTokenData::default(),
            );

            tokens.push(payment);
        }

        let op_sender = managed_address!(&Address::from(&USER_ADDRESS.eval_to_array()));
        let data: OperationData<StaticApi> = OperationData::new(1, op_sender, Option::None);

        (tokens, data)
    }

    fn get_operation_hash(&mut self, operation: &Operation<StaticApi>) -> ManagedBuffer<StaticApi> {
        let mut serialized_operation: ManagedBuffer<StaticApi> = ManagedBuffer::new();
        let _ = operation.top_encode(&mut serialized_operation);
        let sha256 = sha256(&serialized_operation.to_vec());

        ManagedBuffer::new_from_bytes(&sha256)
    }
}

#[test]
fn test_deploy() {
    let mut state = EnshrineTestState::new();

    state.propose_setup_contracts(false);
}

#[test]
fn test_sovereign_prefix_no_prefix() {
    let mut state = EnshrineTestState::new();
    let token_vec = Vec::from([NFT_TOKEN_ID, CROWD_TOKEN_ID]);

    state.propose_setup_contracts(false);
    state.propose_register_operation(&token_vec);
    state.propose_execute_operation(None, &token_vec);
}

#[test]
fn test_sovereign_prefix_has_prefix() {
    let mut state = EnshrineTestState::new();
    let token_vec = Vec::from([PREFIX_NFT_TOKEN_ID, CROWD_TOKEN_ID]);

    state.propose_setup_contracts(false);
    state.propose_register_operation(&token_vec);
    state.propose_execute_operation(None, &token_vec);
}

#[test]
fn test_register_tokens_insufficient_funds() {
    let mut state = EnshrineTestState::new();
    let token_vec = Vec::from([PREFIX_NFT_TOKEN_ID, CROWD_TOKEN_ID]);
    let code = 10u64;
    let error_message = "insufficient funds";
    let payment_amount = BigUint::from(DEFAULT_ISSUE_COST * token_vec.len() as u64);
    let payment = EsdtTokenPayment::new(WEGLD_IDENTIFIER.into(), 0, payment_amount);

    state.propose_setup_contracts(false);
    state.propose_register_tokens(
        &USER_ADDRESS,
        payment,
        token_vec,
        Some(ErrorStatus {
            code,
            error_message,
        }),
    );
}

#[test]
fn test_register_tokens_wrong_token_as_fee() {
    let mut state = EnshrineTestState::new();
    let token_vec = Vec::from([PREFIX_NFT_TOKEN_ID, CROWD_TOKEN_ID]);
    let code = 4u64;
    let error_message = "WEGLD is the only token accepted as register fee";
    let payment_amount = BigUint::from(DEFAULT_ISSUE_COST * token_vec.len() as u64);
    let payment = EsdtTokenPayment::new(CROWD_TOKEN_ID.into(), 0, payment_amount);

    state.propose_setup_contracts(false);
    state.propose_register_tokens(
        &ENSHRINE_ESDT_OWNER_ADDRESS,
        payment,
        token_vec,
        Some(ErrorStatus {
            code,
            error_message,
        }),
    );
}

#[test]
fn test_register_tokens() {
    let mut state = EnshrineTestState::new();
    let token_vec = Vec::from([PREFIX_NFT_TOKEN_ID, CROWD_TOKEN_ID]);
    let payment_amount = BigUint::from(DEFAULT_ISSUE_COST * token_vec.len() as u64);
    let payment = EsdtTokenPayment::new(WEGLD_IDENTIFIER.into(), 0, payment_amount);

    state.propose_setup_contracts(false);
    state.propose_register_tokens(&ENSHRINE_ESDT_OWNER_ADDRESS, payment, token_vec, None);
    state
        .world
        .check_account(ENSHRINE_ESDT_OWNER_ADDRESS)
        .esdt_balance(WEGLD_IDENTIFIER, BigUint::zero());
}

#[test]
fn test_register_tokens_insufficient_wegld() {
    let mut state = EnshrineTestState::new();
    let token_vec = Vec::from([
        NFT_TOKEN_ID,
        PREFIX_NFT_TOKEN_ID,
        FUNGIBLE_TOKEN_ID,
        CROWD_TOKEN_ID,
    ]);
    let code = 4u64;
    let error_message = "WEGLD fee amount is not met";
    let payment_amount = BigUint::from(DEFAULT_ISSUE_COST + token_vec.len() as u64);
    let payment = EsdtTokenPayment::new(WEGLD_IDENTIFIER.into(), 0, payment_amount);

    state.propose_setup_contracts(false);
    state.propose_register_tokens(
        &ENSHRINE_ESDT_OWNER_ADDRESS,
        payment,
        token_vec,
        Some(ErrorStatus {
            code,
            error_message,
        }),
    );
}
