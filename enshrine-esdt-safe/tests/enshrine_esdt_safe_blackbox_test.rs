use bls_signature::BlsSignature;
use enshrine_esdt_safe::{enshrine_esdt_safe_proxy, token_handler_proxy};
use fee_market::fee_market_proxy::{self, FeeStruct, FeeType};
use header_verifier::header_verifier_proxy;
use multiversx_sc::codec::TopEncode;
use multiversx_sc::imports::{MultiValue3, OptionalValue};
use multiversx_sc::types::{
    Address, BigUint, EsdtTokenData, EsdtTokenPayment, ManagedBuffer, ManagedByteArray, ManagedVec,
    MultiValueEncoded, TestAddress, TestSCAddress, TestTokenIdentifier, TokenIdentifier,
};
use multiversx_sc_scenario::api::StaticApi;
use multiversx_sc_scenario::multiversx_chain_vm::crypto_functions::sha256;
use multiversx_sc_scenario::{imports::MxscPath, ScenarioWorld};
use multiversx_sc_scenario::{managed_address, ExpectError, ScenarioTxRun};
use transaction::{GasLimit, Operation, OperationData, OperationEsdtPayment};
use utils::PaymentsVec;

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

const FEE_MARKET_ADDRESS: TestSCAddress = TestSCAddress::new("fee-market");
const FEE_MARKET_CODE_PATH: MxscPath = MxscPath::new("../fee-market/output/fee-market.mxsc.json");

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

type OptionalTransferData<M> =
    OptionalValue<MultiValue3<GasLimit, ManagedBuffer<M>, ManagedVec<M, ManagedBuffer<M>>>>;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(ENSHRINE_ESDT_CODE_PATH, enshrine_esdt_safe::ContractBuilder);
    blockchain.register_contract(HEADER_VERIFIER_CODE_PATH, header_verifier::ContractBuilder);
    blockchain.register_contract(TOKEN_HANDLER_CODE_PATH, token_handler::ContractBuilder);
    blockchain.register_contract(FEE_MARKET_CODE_PATH, fee_market::ContractBuilder);

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

    fn deploy_fee_market_contract(
        &mut self,
        fee_struct: Option<FeeStruct<StaticApi>>,
    ) -> &mut Self {
        self.world
            .tx()
            .from(ENSHRINE_ESDT_OWNER_ADDRESS)
            .typed(fee_market_proxy::FeeMarketProxy)
            .init(ENSHRINE_ESDT_ADDRESS, fee_struct)
            .code(FEE_MARKET_CODE_PATH)
            .new_address(FEE_MARKET_ADDRESS)
            .run();

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
            .init()
            .code(TOKEN_HANDLER_CODE_PATH)
            .new_address(TOKEN_HANDLER_ADDRESS)
            .run();

        self
    }

    fn propose_setup_contracts(
        &mut self,
        is_sovereign_chain: bool,
        fee_struct: Option<&FeeStruct<StaticApi>>,
    ) -> &mut Self {
        self.deploy_enshrine_esdt_contract(
            is_sovereign_chain,
            Some(TokenIdentifier::from(WEGLD_IDENTIFIER)),
            Some(SOVEREIGN_TOKEN_PREFIX.into()),
        );
        self.deploy_header_verifier_contract();
        self.deploy_token_handler_contract();
        self.deploy_fee_market_contract(fee_struct.cloned());

        self.propose_set_header_verifier_address();
        self.propose_register_fee_market_address();

        self
    }

    fn propose_set_fee(
        &mut self,
        fee_struct: Option<&FeeStruct<StaticApi>>,
        error_status: Option<ErrorStatus>,
    ) -> &mut Self {
        if let Some(fee) = fee_struct {
            self.propose_add_fee_token(fee, error_status);
        }

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

    fn propose_register_fee_market_address(&mut self) {
        self.world
            .tx()
            .from(ENSHRINE_ESDT_OWNER_ADDRESS)
            .to(ENSHRINE_ESDT_ADDRESS)
            .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
            .set_fee_market_address(FEE_MARKET_ADDRESS)
            .run();
    }

    fn propose_add_token_to_whitelist(
        &mut self,
        tokens: MultiValueEncoded<StaticApi, TokenIdentifier<StaticApi>>,
    ) {
        self.world
            .tx()
            .from(ENSHRINE_ESDT_OWNER_ADDRESS)
            .to(ENSHRINE_ESDT_ADDRESS)
            .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
            .add_tokens_to_whitelist(tokens)
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
                .from(*sender)
                .to(ENSHRINE_ESDT_ADDRESS)
                .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
                .register_new_token_id(managed_token_ids)
                .returns(ExpectError(status.code, status.error_message))
                .esdt(fee_payment)
                .run(),
            None => self
                .world
                .tx()
                .from(*sender)
                .to(ENSHRINE_ESDT_ADDRESS)
                .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
                .register_new_token_id(managed_token_ids)
                .esdt(fee_payment)
                .run(),
        }
    }

    fn propose_deposit(
        &mut self,
        from: TestAddress,
        to: TestAddress,
        payment: PaymentsVec<StaticApi>,
        deposit_args: OptionalTransferData<StaticApi>,
        error_status: Option<ErrorStatus>,
    ) {
        match error_status {
            Some(status) => self
                .world
                .tx()
                .from(from)
                .to(ENSHRINE_ESDT_ADDRESS)
                .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
                .deposit(to, deposit_args)
                .payment(payment)
                .returns(ExpectError(status.code, status.error_message))
                .run(),
            None => self
                .world
                .tx()
                .from(from)
                .to(ENSHRINE_ESDT_ADDRESS)
                .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
                .deposit(to, deposit_args)
                .payment(payment)
                .run(),
        }
    }

    fn propose_add_fee_token(
        &mut self,
        fee_struct: &FeeStruct<StaticApi>,
        error_status: Option<ErrorStatus>,
    ) {
        match error_status {
            Some(error) => self
                .world
                .tx()
                .from(ENSHRINE_ESDT_OWNER_ADDRESS)
                .to(FEE_MARKET_ADDRESS)
                .typed(fee_market_proxy::FeeMarketProxy)
                .set_fee(fee_struct)
                .returns(ExpectError(error.code, error.error_message))
                .run(),
            None => self
                .world
                .tx()
                .from(ENSHRINE_ESDT_OWNER_ADDRESS)
                .to(FEE_MARKET_ADDRESS)
                .typed(fee_market_proxy::FeeMarketProxy)
                .set_fee(fee_struct)
                .run(),
        }
    }

    fn propose_set_max_user_tx_gas_limit(&mut self, max_gas_limit: GasLimit) {
        self.world
            .tx()
            .from(ENSHRINE_ESDT_OWNER_ADDRESS)
            .to(ENSHRINE_ESDT_ADDRESS)
            .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
            .set_max_user_tx_gas_limit(max_gas_limit)
            .run();
    }

    fn propose_set_banned_endpoint(&mut self, endpoint_name: ManagedBuffer<StaticApi>) {
        self.world
            .tx()
            .from(ENSHRINE_ESDT_OWNER_ADDRESS)
            .to(ENSHRINE_ESDT_ADDRESS)
            .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
            .set_banned_endpoint(endpoint_name)
            .run();
    }

    fn propose_whitelist_enshrine_esdt(&mut self) {
        self.world
            .tx()
            .from(ENSHRINE_ESDT_OWNER_ADDRESS)
            .to(TOKEN_HANDLER_ADDRESS)
            .typed(token_handler_proxy::TokenHandlerProxy)
            .whitelist_enshrine_esdt(ENSHRINE_ESDT_ADDRESS)
            .run();
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
            let payment: OperationEsdtPayment<StaticApi> =
                OperationEsdtPayment::new((*token_id).into(), 1, EsdtTokenData::default());

            tokens.push(payment);
        }

        let op_sender = managed_address!(&Address::from(&USER_ADDRESS.eval_to_array()));
        let data: OperationData<StaticApi> = OperationData::new(1, op_sender, Option::None);

        (tokens, data)
    }

    fn setup_transfer_data(
        &mut self,
        gas_limit: GasLimit,
        function: ManagedBuffer<StaticApi>,
        args: ManagedVec<StaticApi, ManagedBuffer<StaticApi>>,
    ) -> OptionalTransferData<StaticApi> {
        OptionalValue::Some((gas_limit, function, args).into())
    }

    fn get_operation_hash(&mut self, operation: &Operation<StaticApi>) -> ManagedBuffer<StaticApi> {
        let mut serialized_operation: ManagedBuffer<StaticApi> = ManagedBuffer::new();
        let _ = operation.top_encode(&mut serialized_operation);
        let sha256 = sha256(&serialized_operation.to_vec());

        ManagedBuffer::new_from_bytes(&sha256)
    }

    // TODO: add match for fee type
    fn setup_fee_struct(
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
}

#[test]
fn test_deploy() {
    let mut state = EnshrineTestState::new();

    state.propose_setup_contracts(false, None);
}

#[test]
fn test_sovereign_prefix_no_prefix() {
    let mut state = EnshrineTestState::new();
    let token_vec = Vec::from([NFT_TOKEN_ID, CROWD_TOKEN_ID]);
    let error_status = Some(ErrorStatus {
        code: 10,
        error_message: "action is not allowed",
    });

    state.propose_setup_contracts(false, None);
    state.propose_register_operation(&token_vec);
    state.propose_whitelist_enshrine_esdt();
    state.propose_execute_operation(error_status, &token_vec);
}

#[test]
fn test_sovereign_prefix_has_prefix() {
    let mut state = EnshrineTestState::new();
    let token_vec = Vec::from([PREFIX_NFT_TOKEN_ID, CROWD_TOKEN_ID]);

    state.propose_setup_contracts(false, None);
    state.propose_register_operation(&token_vec);
    state.propose_whitelist_enshrine_esdt();
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

    state.propose_setup_contracts(false, None);
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

    state.propose_setup_contracts(false, None);
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

    state.propose_setup_contracts(false, None);
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

    state.propose_setup_contracts(false, None);
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
fn test_deposit_no_fee() {
    let mut state = EnshrineTestState::new();
    let amount = BigUint::from(10000u64);
    let wegld_payment = EsdtTokenPayment::new(WEGLD_IDENTIFIER.into(), 0, amount.clone());
    let mut payments = PaymentsVec::new();

    payments.push(wegld_payment);

    state.propose_setup_contracts(false, None);
    state.propose_set_fee(None, None);
    state.propose_deposit(
        ENSHRINE_ESDT_OWNER_ADDRESS,
        USER_ADDRESS,
        payments,
        OptionalValue::None,
        None,
    );
}

#[test]
fn test_deposit_token_nothing_to_transfer_fee_enabled() {
    let mut state = EnshrineTestState::new();
    let amount = BigUint::from(10000u64);
    let wegld_payment = EsdtTokenPayment::new(WEGLD_IDENTIFIER.into(), 0, amount.clone());
    let mut payments = PaymentsVec::new();
    let error_status = ErrorStatus {
        code: 4,
        error_message: "Nothing to transfer",
    };

    let fee_amount_per_transfer = BigUint::from(100u32);
    let fee_amount_per_gas = BigUint::from(100u32);

    let fee_struct = state.setup_fee_struct(
        WEGLD_IDENTIFIER,
        &fee_amount_per_transfer,
        &fee_amount_per_gas,
    );

    payments.push(wegld_payment);

    state.propose_setup_contracts(false, Some(&fee_struct));
    state.propose_set_fee(Some(&fee_struct), None);
    state.propose_deposit(
        ENSHRINE_ESDT_OWNER_ADDRESS,
        USER_ADDRESS,
        payments,
        OptionalValue::None,
        Some(error_status),
    );
}

#[test]
fn test_deposit_max_transfers_exceeded() {
    let mut state = EnshrineTestState::new();
    let amount = BigUint::from(10000u64);
    let wegld_payment = EsdtTokenPayment::new(WEGLD_IDENTIFIER.into(), 0, amount.clone());
    let mut payments = PaymentsVec::new();
    let error_status = ErrorStatus {
        code: 4,
        error_message: "Too many tokens",
    };

    payments.extend(std::iter::repeat(wegld_payment).take(11));

    state.propose_setup_contracts(false, None);
    state.propose_deposit(
        ENSHRINE_ESDT_OWNER_ADDRESS,
        USER_ADDRESS,
        payments,
        OptionalValue::None,
        Some(error_status),
    );
}

#[test]
fn test_deposit_no_transfer_data() {
    let mut state = EnshrineTestState::new();
    let amount = BigUint::from(10000u64);
    let wegld_payment = EsdtTokenPayment::new(WEGLD_IDENTIFIER.into(), 0, amount.clone());
    let fungible_payment = EsdtTokenPayment::new(FUNGIBLE_TOKEN_ID.into(), 0, amount.clone());
    let crowd_payment = EsdtTokenPayment::new(CROWD_TOKEN_ID.into(), 0, amount.clone());
    let mut payments = PaymentsVec::new();
    let mut tokens_whitelist = MultiValueEncoded::new();
    tokens_whitelist.push(WEGLD_IDENTIFIER.into());
    tokens_whitelist.push(CROWD_TOKEN_ID.into());

    payments.push(wegld_payment);
    payments.push(fungible_payment);
    payments.push(crowd_payment);

    let fee_amount_per_transfer = BigUint::from(100u32);
    let fee_amount_per_gas = BigUint::from(100u32);

    let fee_struct = state.setup_fee_struct(
        WEGLD_IDENTIFIER,
        &fee_amount_per_transfer,
        &fee_amount_per_gas,
    );

    state.propose_setup_contracts(false, Some(&fee_struct));
    state.propose_add_token_to_whitelist(tokens_whitelist);
    state.propose_set_fee(Some(&fee_struct), None);
    state.propose_deposit(
        ENSHRINE_ESDT_OWNER_ADDRESS,
        USER_ADDRESS,
        payments,
        OptionalValue::None,
        None,
    );

    let expected_wegld_amount = BigUint::from(WEGLD_BALANCE) - fee_amount_per_transfer;
    let expected_crowd_amount = BigUint::from(WEGLD_BALANCE) - &amount;

    state
        .world
        .check_account(ENSHRINE_ESDT_OWNER_ADDRESS)
        .esdt_balance(WEGLD_IDENTIFIER, &expected_wegld_amount);

    state
        .world
        .check_account(ENSHRINE_ESDT_OWNER_ADDRESS)
        .esdt_balance(FUNGIBLE_TOKEN_ID, BigUint::from(WEGLD_BALANCE));

    state
        .world
        .check_account(ENSHRINE_ESDT_OWNER_ADDRESS)
        .esdt_balance(CROWD_TOKEN_ID, &expected_crowd_amount);
}

#[test]
fn test_deposit_with_transfer_data_gas_limit_too_high() {
    let mut state = EnshrineTestState::new();
    let amount = BigUint::from(10000u64);
    let wegld_payment = EsdtTokenPayment::new(WEGLD_IDENTIFIER.into(), 0, amount.clone());
    let crowd_payment = EsdtTokenPayment::new(CROWD_TOKEN_ID.into(), 0, amount);
    let mut payments = PaymentsVec::new();
    let gas_limit = 1000000000000000000;
    let function = ManagedBuffer::from("some_function");
    let arg = ManagedBuffer::from("arg");
    let mut args = ManagedVec::new();
    args.push(arg);

    let transfer_data = state.setup_transfer_data(gas_limit, function, args);

    payments.push(wegld_payment);
    payments.push(crowd_payment);

    let error_status = ErrorStatus {
        code: 4,
        error_message: "Gas limit too high",
    };

    state.propose_setup_contracts(false, None);
    state.propose_deposit(
        ENSHRINE_ESDT_OWNER_ADDRESS,
        USER_ADDRESS,
        payments,
        transfer_data,
        Some(error_status),
    );
}

#[test]
fn test_deposit_with_transfer_data_banned_endpoint() {
    let mut state = EnshrineTestState::new();
    let amount = BigUint::from(10000u64);
    let wegld_payment = EsdtTokenPayment::new(WEGLD_IDENTIFIER.into(), 0, amount.clone());
    let crowd_payment = EsdtTokenPayment::new(CROWD_TOKEN_ID.into(), 0, amount);
    let mut payments = PaymentsVec::new();
    let gas_limit = 1000000000;
    let function = ManagedBuffer::from("some_function");
    let arg = ManagedBuffer::from("arg");
    let mut args = ManagedVec::new();
    args.push(arg);

    let transfer_data = state.setup_transfer_data(gas_limit, function.clone(), args);

    payments.push(wegld_payment);
    payments.push(crowd_payment);

    let error_status = ErrorStatus {
        code: 4,
        error_message: "Banned endpoint name",
    };

    // TODO: idk if it supposed to be None
    state.propose_setup_contracts(false, None);
    state.propose_set_max_user_tx_gas_limit(gas_limit);
    state.propose_set_banned_endpoint(function);
    state.propose_deposit(
        ENSHRINE_ESDT_OWNER_ADDRESS,
        USER_ADDRESS,
        payments,
        transfer_data,
        Some(error_status),
    );
}

#[test]
fn test_deposit_with_transfer_data_enough_for_fee() {
    let mut state = EnshrineTestState::new();
    let amount = BigUint::from(1000000000000000u128);
    let wegld_payment = EsdtTokenPayment::new(WEGLD_IDENTIFIER.into(), 0, amount.clone());
    let fungible_payment = EsdtTokenPayment::new(FUNGIBLE_TOKEN_ID.into(), 0, amount.clone());
    let crowd_payment = EsdtTokenPayment::new(CROWD_TOKEN_ID.into(), 0, amount.clone());
    let mut payments = PaymentsVec::new();
    let gas_limit = 1000000000;
    let function = ManagedBuffer::from("some_function");
    let arg = ManagedBuffer::from("arg");
    let mut args = ManagedVec::new();
    args.push(arg);

    let transfer_data = state.setup_transfer_data(gas_limit, function, args);

    let expected_crowd_amount = BigUint::from(WEGLD_BALANCE) - &wegld_payment.amount;
    let expected_fungible_amount = BigUint::from(WEGLD_BALANCE) - &fungible_payment.amount;

    payments.push(wegld_payment);
    payments.push(fungible_payment);
    payments.push(crowd_payment);

    let fee_amount_per_transfer = BigUint::from(100u32);
    let fee_amount_per_gas = BigUint::from(100u32);

    let fee_struct = state.setup_fee_struct(
        WEGLD_IDENTIFIER,
        &fee_amount_per_transfer,
        &fee_amount_per_gas,
    );

    state.propose_setup_contracts(false, Some(&fee_struct));
    state.propose_set_max_user_tx_gas_limit(gas_limit);
    state.propose_set_fee(Some(&fee_struct), None);
    state.propose_deposit(
        ENSHRINE_ESDT_OWNER_ADDRESS,
        USER_ADDRESS,
        payments,
        transfer_data,
        None,
    );

    let fee = fee_amount_per_transfer * BigUint::from(2u32)
        + BigUint::from(gas_limit) * fee_amount_per_gas;
    let expected_wegld_amount = BigUint::from(WEGLD_BALANCE) - fee;

    state
        .world
        .check_account(ENSHRINE_ESDT_OWNER_ADDRESS)
        .esdt_balance(WEGLD_IDENTIFIER, &expected_wegld_amount);

    state
        .world
        .check_account(ENSHRINE_ESDT_OWNER_ADDRESS)
        .esdt_balance(FUNGIBLE_TOKEN_ID, &expected_fungible_amount);

    state
        .world
        .check_account(ENSHRINE_ESDT_OWNER_ADDRESS)
        .esdt_balance(CROWD_TOKEN_ID, &expected_crowd_amount);
}

#[test]
fn test_deposit_with_transfer_data_not_enough_for_fee() {
    let mut state = EnshrineTestState::new();
    let amount = BigUint::from(100000000000000000u128);
    let wegld_payment = EsdtTokenPayment::new(WEGLD_IDENTIFIER.into(), 0, amount.clone());
    let fungible_payment = EsdtTokenPayment::new(FUNGIBLE_TOKEN_ID.into(), 0, amount.clone());
    let crowd_payment = EsdtTokenPayment::new(CROWD_TOKEN_ID.into(), 0, amount.clone());
    let mut payments = PaymentsVec::new();
    let gas_limit = 1000000000000000;
    let function = ManagedBuffer::from("some_function");
    let arg = ManagedBuffer::from("arg");
    let mut args = ManagedVec::new();
    args.push(arg);

    let error_status = ErrorStatus {
        code: 4,
        error_message: "Payment does not cover fee",
    };

    let transfer_data = state.setup_transfer_data(gas_limit, function, args);

    payments.push(wegld_payment);
    payments.push(fungible_payment);
    payments.push(crowd_payment);

    let fee_amount_per_transfer = BigUint::from(100u32);
    let fee_amount_per_gas = BigUint::from(100u32);

    let fee_struct = state.setup_fee_struct(
        WEGLD_IDENTIFIER,
        &fee_amount_per_transfer,
        &fee_amount_per_gas,
    );

    state.propose_setup_contracts(false, Some(&fee_struct));
    state.propose_set_max_user_tx_gas_limit(gas_limit);
    state.propose_set_fee(Some(&fee_struct), None);
    state.propose_deposit(
        ENSHRINE_ESDT_OWNER_ADDRESS,
        USER_ADDRESS,
        payments,
        transfer_data,
        Some(error_status),
    );
}

#[test]
fn test_deposit_refund_non_whitelisted_tokens_fee_disabled() {
    let mut state = EnshrineTestState::new();
    let mut payments = PaymentsVec::new();
    let amount = BigUint::from(100000000000000000u128);
    let wegld_payment = EsdtTokenPayment::new(WEGLD_IDENTIFIER.into(), 0, amount.clone());
    let fungible_payment = EsdtTokenPayment::new(FUNGIBLE_TOKEN_ID.into(), 0, amount.clone());
    let crowd_payment = EsdtTokenPayment::new(CROWD_TOKEN_ID.into(), 0, amount.clone());
    let mut token_whitelist = MultiValueEncoded::new();
    token_whitelist.push(NFT_TOKEN_ID.into());

    payments.push(wegld_payment);
    payments.push(fungible_payment);
    payments.push(crowd_payment);

    state.propose_setup_contracts(false, None);
    state.propose_add_token_to_whitelist(token_whitelist);
    state.propose_deposit(
        ENSHRINE_ESDT_OWNER_ADDRESS,
        USER_ADDRESS,
        payments,
        OptionalValue::None,
        None,
    );

    let expected_amount = BigUint::from(WEGLD_BALANCE);

    state
        .world
        .check_account(ENSHRINE_ESDT_OWNER_ADDRESS)
        .esdt_balance(FUNGIBLE_TOKEN_ID, &expected_amount);

    state
        .world
        .check_account(ENSHRINE_ESDT_OWNER_ADDRESS)
        .esdt_balance(CROWD_TOKEN_ID, &expected_amount);
}

#[test]
fn test_deposit_refund_non_whitelisted_tokens_fee_enabled() {
    let mut state = EnshrineTestState::new();
    let mut payments = PaymentsVec::new();
    let amount = BigUint::from(100000000000000000u128);
    let wegld_payment = EsdtTokenPayment::new(WEGLD_IDENTIFIER.into(), 0, amount.clone());
    let fungible_payment = EsdtTokenPayment::new(FUNGIBLE_TOKEN_ID.into(), 0, amount.clone());
    let crowd_payment = EsdtTokenPayment::new(CROWD_TOKEN_ID.into(), 0, amount.clone());
    let mut token_whitelist = MultiValueEncoded::new();
    token_whitelist.push(NFT_TOKEN_ID.into());

    payments.push(wegld_payment);
    payments.push(fungible_payment);
    payments.push(crowd_payment);

    let fee_amount_per_transfer = BigUint::from(100u32);
    let fee_amount_per_gas = BigUint::from(100u32);

    let fee_struct = state.setup_fee_struct(
        WEGLD_IDENTIFIER,
        &fee_amount_per_transfer,
        &fee_amount_per_gas,
    );

    state.propose_setup_contracts(false, Some(&fee_struct));
    state.propose_add_token_to_whitelist(token_whitelist);
    state.propose_set_fee(Some(&fee_struct), None);
    state.propose_deposit(
        ENSHRINE_ESDT_OWNER_ADDRESS,
        USER_ADDRESS,
        payments,
        OptionalValue::None,
        None,
    );

    let expected_amount = BigUint::from(WEGLD_BALANCE);

    state
        .world
        .check_account(ENSHRINE_ESDT_OWNER_ADDRESS)
        .esdt_balance(FUNGIBLE_TOKEN_ID, &expected_amount);

    state
        .world
        .check_account(ENSHRINE_ESDT_OWNER_ADDRESS)
        .esdt_balance(CROWD_TOKEN_ID, &expected_amount);
}
