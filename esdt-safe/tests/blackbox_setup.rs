use bls_signature::BlsSignature;
use esdt_safe::esdt_safe_proxy::{self};
use fee_market::fee_market_proxy::{self, FeeStruct};
use header_verifier::header_verifier_proxy;
use multiversx_sc::codec::TopEncode;
use multiversx_sc::types::{
    Address, EsdtTokenData, ManagedByteArray, MultiValueEncoded, TestTokenIdentifier,
};
use multiversx_sc::{
    imports::{MultiValue3, OptionalValue},
    types::{BigUint, EsdtTokenPayment, ManagedBuffer, ManagedVec, TestAddress, TestSCAddress},
};
use multiversx_sc_scenario::multiversx_chain_vm::crypto_functions::sha256;
use multiversx_sc_scenario::{api::StaticApi, imports::MxscPath, ScenarioTxRun, ScenarioWorld};
use multiversx_sc_scenario::{managed_address, ReturnsHandledOrError};
use transaction::{Operation, OperationData, OperationEsdtPayment};

const BRIDGE_ADDRESS: TestSCAddress = TestSCAddress::new("bridge");
const BRIDGE_CODE_PATH: MxscPath = MxscPath::new("output/esdt-safe.mxsc.json");
const BRIDGE_OWNER_ADDRESS: TestAddress = TestAddress::new("bridge_owner");

const FEE_MARKET_ADDRESS: TestSCAddress = TestSCAddress::new("fee_market");
const FEE_MARKET_CODE_PATH: MxscPath = MxscPath::new("../fee-market/output/fee-market.mxsc.json");

const HEADER_VERIFIER_ADDRESS: TestSCAddress = TestSCAddress::new("header_verifier");
const HEADER_VERIFIER_CODE_PATH: MxscPath =
    MxscPath::new("../header-verifier/output/header-verifier.mxsc.json");

pub const USER_ADDRESS: TestAddress = TestAddress::new("user");
const RECEIVER_ADDRESS: TestAddress = TestAddress::new("receiver");

const EGLD_BALANCE: u64 = 1_000_000_000_000_000_000; // 1 EGLD
const EGLD_DEPOSIT: u64 = 100_000_000_000_000_000; // 0.1 EGLD
pub const TOKEN_BALANCE: u64 = 100_000;

const NFT_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("NFT-123456");
pub const FUNGIBLE_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("CROWD-123456");

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(BRIDGE_CODE_PATH, esdt_safe::ContractBuilder);
    blockchain.register_contract(FEE_MARKET_CODE_PATH, fee_market::ContractBuilder);
    blockchain.register_contract(HEADER_VERIFIER_CODE_PATH, header_verifier::ContractBuilder);

    blockchain
}

pub struct BridgeTestState {
    pub world: ScenarioWorld,
}

impl BridgeTestState {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut world = world();

        world
            .account(BRIDGE_OWNER_ADDRESS)
            .esdt_nft_balance(NFT_TOKEN_ID, 1, TOKEN_BALANCE, ManagedBuffer::new())
            .esdt_balance(FUNGIBLE_TOKEN_ID, TOKEN_BALANCE)
            .nonce(1)
            .balance(EGLD_BALANCE);

        world
            .account(USER_ADDRESS)
            .esdt_nft_balance(NFT_TOKEN_ID, 1, TOKEN_BALANCE, ManagedBuffer::new())
            .esdt_balance(FUNGIBLE_TOKEN_ID, TOKEN_BALANCE)
            .balance(EGLD_BALANCE)
            .nonce(1);

        world.account(RECEIVER_ADDRESS).nonce(1);

        Self { world }
    }

    pub fn deploy_bridge_contract(&mut self, is_sovereign_chain: bool) -> &mut Self {
        self.world
            .tx()
            .from(BRIDGE_OWNER_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .init(is_sovereign_chain)
            .code(BRIDGE_CODE_PATH)
            .new_address(BRIDGE_ADDRESS)
            .run();

        self.deploy_fee_market_contract();
        self.propose_set_fee_market_address();
        self.propose_set_unpaused();

        self
    }

    pub fn deploy_fee_market_contract(&mut self) -> &mut Self {
        let fee_struct: Option<FeeStruct<StaticApi>> = None;

        self.world
            .tx()
            .from(BRIDGE_OWNER_ADDRESS)
            .typed(fee_market_proxy::FeeMarketProxy)
            .init(BRIDGE_ADDRESS, fee_struct)
            .code(FEE_MARKET_CODE_PATH)
            .new_address(FEE_MARKET_ADDRESS)
            .run();

        self
    }

    pub fn deploy_header_verifier_contract(&mut self) -> &mut Self {
        let bls_pub_key: ManagedBuffer<StaticApi> = ManagedBuffer::new();
        let mut bls_pub_keys = MultiValueEncoded::new();
        bls_pub_keys.push(bls_pub_key);

        self.world
            .tx()
            .from(BRIDGE_OWNER_ADDRESS)
            .typed(header_verifier_proxy::HeaderverifierProxy)
            .init(bls_pub_keys)
            .code(HEADER_VERIFIER_CODE_PATH)
            .new_address(HEADER_VERIFIER_ADDRESS)
            .run();

        self
    }

    pub fn propose_set_fee_market_address(&mut self) {
        self.world
            .tx()
            .from(BRIDGE_OWNER_ADDRESS)
            .to(BRIDGE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .set_fee_market_address(FEE_MARKET_ADDRESS)
            .run();
    }

    pub fn propose_set_header_verifier_address(&mut self) {
        self.world
            .tx()
            .from(BRIDGE_OWNER_ADDRESS)
            .to(BRIDGE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .set_header_verifier_address(HEADER_VERIFIER_ADDRESS)
            .run();
    }

    pub fn propose_egld_deposit_and_expect_err(&mut self, err_message: Option<&str>) {
        let transfer_data = OptionalValue::<
            MultiValue3<
                u64,
                ManagedBuffer<StaticApi>,
                ManagedVec<StaticApi, ManagedBuffer<StaticApi>>,
            >,
        >::None;

        let response = self
            .world
            .tx()
            .from(USER_ADDRESS)
            .to(BRIDGE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .deposit(RECEIVER_ADDRESS, transfer_data)
            .egld(EGLD_DEPOSIT)
            .returns(ReturnsHandledOrError::new())
            .run();

        match response {
            Ok(_) => assert!(
                err_message.is_none(),
                "Transaction was successful, but expected error"
            ),
            Err(error) => assert_eq!(err_message, Some(error.message.as_str())),
        };
    }

    pub fn propose_esdt_deposit(&mut self) {
        let transfer_data = OptionalValue::<
            MultiValue3<
                u64,
                ManagedBuffer<StaticApi>,
                ManagedVec<StaticApi, ManagedBuffer<StaticApi>>,
            >,
        >::None;

        let mut payments = ManagedVec::new();
        let nft_payment = EsdtTokenPayment::new(NFT_TOKEN_ID.into(), 1, BigUint::from(10u64));
        let fungible_payment: EsdtTokenPayment<StaticApi> =
            EsdtTokenPayment::new(FUNGIBLE_TOKEN_ID.into(), 0, BigUint::from(10u64));

        payments.push(fungible_payment);
        payments.push(nft_payment);

        self.world
            .tx()
            .from(USER_ADDRESS)
            .to(BRIDGE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .deposit(RECEIVER_ADDRESS, transfer_data)
            .payment(payments)
            .run();
    }

    pub fn propose_execute_operation_and_expect_err(&mut self, err_message: Option<&str>) {
        let (tokens, data) = self.setup_payments(vec![NFT_TOKEN_ID, FUNGIBLE_TOKEN_ID]);
        let to = managed_address!(&Address::from(&RECEIVER_ADDRESS.eval_to_array()));
        let operation = Operation { to, tokens, data };
        let operation_hash = self.get_operation_hash(&operation);

        let response = self
            .world
            .tx()
            .from(USER_ADDRESS)
            .to(BRIDGE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .execute_operations(operation_hash, operation)
            .returns(ReturnsHandledOrError::new())
            .run();

        match response {
            Ok(_) => assert!(
                err_message.is_none(),
                "Transaction was successful, but expected error"
            ),
            Err(error) => assert_eq!(err_message, Some(error.message.as_str())),
        };
    }

    pub fn propose_execute_operation(&mut self) {
        let (tokens, data) = self.setup_payments(vec![NFT_TOKEN_ID, FUNGIBLE_TOKEN_ID]);
        let to = managed_address!(&Address::from(&RECEIVER_ADDRESS.eval_to_array()));
        let operation = Operation { to, tokens, data };
        let operation_hash = self.get_operation_hash(&operation);
        let hash_of_hashes: ManagedBuffer<StaticApi> =
            ManagedBuffer::from(&sha256(&operation_hash.to_vec()));

        self.world
            .tx()
            .from(USER_ADDRESS)
            .to(BRIDGE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .execute_operations(hash_of_hashes, operation)
            .run();
    }

    pub fn propose_set_unpaused(&mut self) {
        self.world
            .tx()
            .from(BRIDGE_OWNER_ADDRESS)
            .to(BRIDGE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .unpause_endpoint()
            .run();
    }

    pub fn propose_set_esdt_safe_address(&mut self) {
        self.world
            .tx()
            .from(BRIDGE_OWNER_ADDRESS)
            .to(HEADER_VERIFIER_ADDRESS)
            .typed(header_verifier_proxy::HeaderverifierProxy)
            .set_esdt_safe_address(BRIDGE_ADDRESS)
            .run();
    }

    pub fn blacklist_token(&mut self) {
        let mut tokens = MultiValueEncoded::new();
        tokens.push(FUNGIBLE_TOKEN_ID.into());

        self.world
            .tx()
            .from(BRIDGE_OWNER_ADDRESS)
            .to(BRIDGE_ADDRESS)
            .typed(esdt_safe_proxy::EsdtSafeProxy)
            .add_tokens_to_blacklist(tokens)
            .run();
    }

    pub fn propose_register_operation(&mut self) {
        let (tokens, data) = self.setup_payments(vec![NFT_TOKEN_ID, FUNGIBLE_TOKEN_ID]);
        let to = managed_address!(&Address::from(RECEIVER_ADDRESS.eval_to_array()));
        let operation = Operation { to, tokens, data };
        let operation_hash = self.get_operation_hash(&operation);
        let mut operations_hashes = MultiValueEncoded::new();

        operations_hashes.push(operation_hash.clone());

        let mock_signature = self.mock_bls_signature(&operation_hash);
        let hash_of_hashes = ManagedBuffer::new_from_bytes(&sha256(&operation_hash.to_vec()));

        self.world
            .tx()
            .from(BRIDGE_OWNER_ADDRESS)
            .to(HEADER_VERIFIER_ADDRESS)
            .typed(header_verifier_proxy::HeaderverifierProxy)
            .register_bridge_operations(
                mock_signature,
                hash_of_hashes.clone(),
                operations_hashes.clone(),
            )
            .run();
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
                token_data: EsdtTokenData::default(),
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

    fn mock_bls_signature(
        &mut self,
        operation_hash: &ManagedBuffer<StaticApi>,
    ) -> BlsSignature<StaticApi> {
        let byte_arr: &mut [u8; 48] = &mut [0; 48];
        operation_hash.load_to_byte_array(byte_arr);
        let mock_signature: BlsSignature<StaticApi> = ManagedByteArray::new_from_bytes(byte_arr);

        mock_signature
    }
}
