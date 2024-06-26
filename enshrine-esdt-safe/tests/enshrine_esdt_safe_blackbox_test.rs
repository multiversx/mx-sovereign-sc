use enshrine_esdt_safe::enshrine_esdt_safe_proxy;
use multiversx_sc::codec::TopEncode;
use multiversx_sc::types::{
    Address, ManagedBuffer, ManagedVec, TestAddress, TestSCAddress, TestTokenIdentifier,
};
use multiversx_sc_scenario::api::StaticApi;
use multiversx_sc_scenario::multiversx_chain_vm::crypto_functions::sha256;
use multiversx_sc_scenario::{imports::MxscPath, ScenarioWorld};
use multiversx_sc_scenario::{managed_address, ScenarioTxRun};
use transaction::{
    Operation, OperationData, OperationEsdtPayment, StolenFromFrameworkEsdtTokenData,
};

const ENSHRINE_ESDT_ADDRESS: TestSCAddress = TestSCAddress::new("enshrine-esdt");
const ENSHRINE_ESDT_CODE_PATH: MxscPath = MxscPath::new("output/enshrine-esdt-safe.mxsc-json");
const ENSHRINE_ESDT_OWNER_ADDRESS: TestAddress = TestAddress::new("enshrine-esdt-owner");

const ENSHRINE_OWNER_BALANCE: u64 = 100_000_000;
const USER_EGLD_BALANCE: u64 = 100_000_000;

const USER_ADDRESS: TestAddress = TestAddress::new("user");
const RECEIVER_ADDRESS: TestAddress = TestAddress::new("receiver");

const NFT_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("NFT-123456");
const FUNGIBLE_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("CROWD-123456");

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(ENSHRINE_ESDT_CODE_PATH, enshrine_esdt_safe::ContractBuilder);

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

    fn deploy_enshrine_esdt_contract(&mut self, is_sovereign_chain: bool) -> &mut Self {
        self.world
            .tx()
            .from(ENSHRINE_ESDT_OWNER_ADDRESS)
            .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
            .init(is_sovereign_chain)
            .code(ENSHRINE_ESDT_CODE_PATH)
            .new_address(ENSHRINE_ESDT_ADDRESS)
            .run();

        self.propose_set_unpaused();

        self
    }

    fn propose_execute_operation(&mut self) {
        let (tokens, data) = self.setup_payments(vec![NFT_TOKEN_ID, FUNGIBLE_TOKEN_ID]);
        let to = managed_address!(&Address::from(&RECEIVER_ADDRESS.eval_to_array()));
        let operation = Operation { to, tokens, data };
        let operation_hash = self.get_operation_hash(&operation);
        let hash_of_hashes: ManagedBuffer<StaticApi> =
            ManagedBuffer::from(&sha256(&operation_hash.to_vec()));

        self.world
            .tx()
            .from(USER_ADDRESS)
            .to(ENSHRINE_ESDT_ADDRESS)
            .typed(enshrine_esdt_safe_proxy::EnshrineEsdtSafeProxy)
            .execute_operations(hash_of_hashes, operation)
            .run();
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

    state.deploy_enshrine_esdt_contract(false);
}

#[test]
fn test_sovereign_prefix() {
    let mut state = EnshrineTestState::new();

    state.deploy_enshrine_esdt_contract(false);

    state.propose_execute_operation();
}
