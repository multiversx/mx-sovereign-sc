use bls_signature::BlsSignature;
use enshrine_esdt_safe::enshrine_esdt_safe_proxy;
use header_verifier::header_verifier_proxy;
use multiversx_sc::codec::TopEncode;
use multiversx_sc::types::{
    Address, BigUint, EsdtTokenPayment, ManagedBuffer, ManagedByteArray, ManagedVec,
    MultiValueEncoded, TestAddress, TestSCAddress, TestTokenIdentifier, TokenIdentifier,
};
use multiversx_sc_scenario::api::StaticApi;
use multiversx_sc_scenario::multiversx_chain_vm::crypto_functions::sha256;
use multiversx_sc_scenario::{imports::MxscPath, ScenarioWorld};
use multiversx_sc_scenario::{managed_address, ExpectError, ScenarioTxRun};
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

const USER_ADDRESS: TestAddress = TestAddress::new("user");
const RECEIVER_ADDRESS: TestAddress = TestAddress::new("receiver");

const NFT_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("NFT-123456");
const FUNGIBLE_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("CROWD-123456");
const PREFIX_NFT_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("sov-NFT-123456");

const WEGLD_IDENTIFIER: TestTokenIdentifier = TestTokenIdentifier::new("WEGLD-123456");
const SOVEREIGN_TOKEN_PREFIX: &str = "sov-";

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(ENSHRINE_ESDT_CODE_PATH, enshrine_esdt_safe::ContractBuilder);
    blockchain.register_contract(HEADER_VERIFIER_CODE_PATH, header_verifier::ContractBuilder);

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
            .esdt_balance(FUNGIBLE_TOKEN_ID, 100_000)
            .esdt_balance(
                WEGLD_IDENTIFIER,
                BigUint::from(100_000_000_000_000_000 as u128),
            )
            .esdt_nft_balance(NFT_TOKEN_ID, 1, 100_000, ManagedBuffer::new())
            .nonce(1)
            .balance(ENSHRINE_OWNER_BALANCE);

        world
            .account(USER_ADDRESS)
            .esdt_nft_balance(NFT_TOKEN_ID, 1, 100_000, ManagedBuffer::new())
            .esdt_balance(FUNGIBLE_TOKEN_ID, 100_000)
            .balance(USER_EGLD_BALANCE)
            .nonce(1);

        world.account(RECEIVER_ADDRESS).nonce(1);

        Self { world }
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
            .init(is_sovereign_chain, wegld_identifier, sovereign_token_prefix)
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

    fn propose_setup_contracts(&mut self, is_sovereign_chain: bool) -> &mut Self {
        self.deploy_enshrine_esdt_contract(
            is_sovereign_chain,
            Some(TokenIdentifier::from(WEGLD_IDENTIFIER)),
            Some(SOVEREIGN_TOKEN_PREFIX.into()),
        );
        self.deploy_header_verifier_contract();
        self.propose_set_header_verifier_address();

        self
    }

    fn propose_execute_operation(&mut self, has_prefix: bool, error_msg: Option<&str>) {
        let (tokens, data) = if has_prefix {
            self.setup_payments(vec![PREFIX_NFT_TOKEN_ID, FUNGIBLE_TOKEN_ID])
        } else {
            self.setup_payments(vec![NFT_TOKEN_ID, FUNGIBLE_TOKEN_ID])
        };
        let to = managed_address!(&Address::from(&RECEIVER_ADDRESS.eval_to_array()));
        let operation = Operation { to, tokens, data };
        let operation_hash = self.get_operation_hash(&operation);
        let hash_of_hashes: ManagedBuffer<StaticApi> =
            ManagedBuffer::from(&sha256(&operation_hash.to_vec()));

        match error_msg {
            Some(msg) => {
                self.world
                    .tx()
                    .from(USER_ADDRESS)
                    .to(ENSHRINE_ESDT_ADDRESS)
                    .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
                    .execute_operations(hash_of_hashes, operation)
                    .returns(ExpectError(10, msg))
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

    fn propose_register_operation(&mut self, has_prefix: bool) {
        let (tokens, data) = if has_prefix {
            self.setup_payments(vec![PREFIX_NFT_TOKEN_ID, FUNGIBLE_TOKEN_ID])
        } else {
            self.setup_payments(vec![NFT_TOKEN_ID, FUNGIBLE_TOKEN_ID])
        };
        let to = managed_address!(&Address::from(RECEIVER_ADDRESS.eval_to_array()));
        let operation = Operation { to, tokens, data };
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
        token_ids: Vec<TestTokenIdentifier>,
        opt_err_message: Option<&str>,
    ) {
        let mut managed_token_ids: MultiValueEncoded<StaticApi, TokenIdentifier<StaticApi>> =
            MultiValueEncoded::new();

        for token_id in token_ids {
            managed_token_ids.push(TokenIdentifier::from(token_id))
        }

        let wegld_amount = BigUint::from(DEFAULT_ISSUE_COST * managed_token_ids.len() as u64);
        let wegld_payment = EsdtTokenPayment::new(WEGLD_IDENTIFIER.into(), 0, wegld_amount);

        match opt_err_message {
            Some(err_msg) => self
                .world
                .tx()
                .from(ENSHRINE_ESDT_OWNER_ADDRESS)
                .to(ENSHRINE_ESDT_ADDRESS)
                .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
                .register_new_token_id(managed_token_ids)
                .returns(ExpectError(4, err_msg))
                .esdt(wegld_payment)
                .run(),
            None => self
                .world
                .tx()
                .from(ENSHRINE_ESDT_OWNER_ADDRESS)
                .to(ENSHRINE_ESDT_ADDRESS)
                .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
                .register_new_token_id(managed_token_ids)
                .esdt(wegld_payment)
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
        token_ids: Vec<TestTokenIdentifier>,
    ) -> (
        ManagedVec<StaticApi, OperationEsdtPayment<StaticApi>>,
        OperationData<StaticApi>,
    ) {
        let mut tokens: ManagedVec<StaticApi, OperationEsdtPayment<StaticApi>> = ManagedVec::new();

        for token_id in token_ids {
            let payment: OperationEsdtPayment<StaticApi> = OperationEsdtPayment {
                token_identifier: token_id.into(),
                token_nonce: 1,
                token_data: StolenFromFrameworkEsdtTokenData::default(),
            };

            tokens.push(payment);
        }

        let op_sender = managed_address!(&Address::from(&USER_ADDRESS.eval_to_array()));
        let data: OperationData<StaticApi> = OperationData {
            op_nonce: 1,
            op_sender,
            opt_transfer_data: Option::None,
        };

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

    state.propose_setup_contracts(false);
    state.propose_register_operation(false);
    state.propose_execute_operation(false, None);
}

#[test]
fn test_sovereign_prefix_has_prefix() {
    let mut state = EnshrineTestState::new();

    state.propose_setup_contracts(false);
    state.propose_register_operation(true);
    state.propose_execute_operation(true, None);
}

#[test]
fn test_register_tokens_multiple_esdt() {
    let mut state = EnshrineTestState::new();
    let error_message = "incorrect number of ESDT transfers";

    state.propose_setup_contracts(false);
    state.propose_register_tokens(
        Vec::from([PREFIX_NFT_TOKEN_ID, FUNGIBLE_TOKEN_ID]),
        Some(error_message),
    );
}

#[test]
fn test_register_tokens_with_egld() {
    let mut state = EnshrineTestState::new();

    state.propose_setup_contracts(false);
    state.propose_register_tokens(Vec::from([PREFIX_NFT_TOKEN_ID, FUNGIBLE_TOKEN_ID]), None);
}
