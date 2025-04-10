#![no_std]

use header_verifier::{Headerverifier, OperationHashStatus};
use multiversx_sc_scenario::{
    api::StaticApi,
    imports::{
        BigUint, ContractBase, EgldOrEsdtTokenIdentifier, EsdtTokenType, ManagedAddress,
        ManagedBuffer, MultiValue2, MultiValue3, MultiValueEncoded, MxscPath, TestAddress,
        TestSCAddress, TestTokenIdentifier, TokenIdentifier, Vec,
    },
    multiversx_chain_vm::crypto_functions::sha256,
    scenario_model::TxResponseStatus,
    DebugApi, ScenarioTxRun, ScenarioTxWhitebox, ScenarioWorld,
};
use proxies::{
    chain_config_proxy::ChainConfigContractProxy,
    fee_market_proxy::{FeeMarketProxy, FeeStruct},
    header_verifier_proxy::HeaderverifierProxy,
    testing_sc_proxy::TestingScProxy,
};
use structs::configs::SovereignConfig;

use cross_chain::storage::CrossChainStorage;

pub const ESDT_SAFE_ADDRESS: TestSCAddress = TestSCAddress::new("esdt-safe");
pub const FEE_MARKET_ADDRESS: TestSCAddress = TestSCAddress::new("fee-market");
pub const HEADER_VERIFIER_ADDRESS: TestSCAddress = TestSCAddress::new("header-verifier");
pub const TESTING_SC_ADDRESS: TestSCAddress = TestSCAddress::new("testing-sc");
pub const CHAIN_CONFIG_ADDRESS: TestSCAddress = TestSCAddress::new("chain-config");
pub const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");
pub const USER: TestAddress = TestAddress::new("user");

pub const FEE_MARKET_CODE_PATH: MxscPath =
    MxscPath::new("../fee-market/output/fee-market.mxsc.json");
pub const HEADER_VERIFIER_CODE_PATH: MxscPath =
    MxscPath::new("../header-verifier/output/header-verifier.mxsc.json");
pub const CHAIN_CONFIG_CODE_PATH: MxscPath =
    MxscPath::new("../chain-config/output/chain-config.mxsc.json");
pub const TESTING_SC_CODE_PATH: MxscPath =
    MxscPath::new("../testing-sc/output/testing-sc.mxsc.json");

pub const TEST_TOKEN_ONE: &str = "TONE-123456";
pub const TEST_TOKEN_ONE_WITH_PREFIX: &str = "sov-TONE-123456";
pub const TEST_TOKEN_TWO: &str = "TTWO-123456";
pub const FEE_TOKEN: &str = "FEE-123456";

pub const SOV_TO_MVX_TOKEN_STORAGE_KEY: &str = "sovToMxTokenId";
pub const MVX_TO_SOV_TOKEN_STORAGE_KEY: &str = "mxToSovTokenId";
pub const OPERATION_HASH_STATUS_STORAGE_KEY: &str = "operationHashStatus";

pub const ISSUE_COST: u64 = 50_000_000_000_000_000; // 0.05 EGLD
pub const ONE_HUNDRED_MILLION: u32 = 100_000_000;
pub const ONE_HUNDRED_THOUSAND: u32 = 100_000;
pub const OWNER_BALANCE: u128 = 100_000_000_000_000_000_000_000;

pub struct RegisterTokenArgs<'a> {
    pub sov_token_id: TokenIdentifier<StaticApi>,
    pub token_type: EsdtTokenType,
    pub token_display_name: &'a str,
    pub token_ticker: &'a str,
    pub num_decimals: usize,
}

pub struct BaseSetup {
    pub world: ScenarioWorld,
}

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(FEE_MARKET_CODE_PATH, fee_market::ContractBuilder);
    blockchain.register_contract(HEADER_VERIFIER_CODE_PATH, header_verifier::ContractBuilder);
    blockchain.register_contract(CHAIN_CONFIG_CODE_PATH, chain_config::ContractBuilder);
    blockchain.register_contract(TESTING_SC_CODE_PATH, testing_sc::ContractBuilder);

    blockchain
}

impl BaseSetup {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut world = world();

        world
            .account(OWNER_ADDRESS)
            .nonce(1)
            .esdt_balance(
                TokenIdentifier::from(FEE_TOKEN),
                BigUint::from(ONE_HUNDRED_MILLION),
            )
            .esdt_balance(
                TokenIdentifier::from(TEST_TOKEN_ONE),
                BigUint::from(ONE_HUNDRED_MILLION),
            )
            .esdt_balance(
                TokenIdentifier::from(TEST_TOKEN_TWO),
                BigUint::from(ONE_HUNDRED_MILLION),
            )
            .balance(BigUint::from(OWNER_BALANCE));

        world
            .account(USER)
            .nonce(1)
            .esdt_balance(
                TokenIdentifier::from(TEST_TOKEN_ONE),
                BigUint::from(ONE_HUNDRED_MILLION),
            )
            .balance(BigUint::from(OWNER_BALANCE));
        Self { world }
    }

    pub fn deploy_fee_market(&mut self, fee: Option<FeeStruct<StaticApi>>) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(FeeMarketProxy)
            .init(ESDT_SAFE_ADDRESS, fee)
            .code(FEE_MARKET_CODE_PATH)
            .new_address(FEE_MARKET_ADDRESS)
            .run();

        self
    }

    pub fn deploy_testing_sc(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(TestingScProxy)
            .init()
            .code(TESTING_SC_CODE_PATH)
            .new_address(TESTING_SC_ADDRESS)
            .run();

        self
    }

    pub fn deploy_header_verifier(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(HeaderverifierProxy)
            .init()
            .code(HEADER_VERIFIER_CODE_PATH)
            .new_address(HEADER_VERIFIER_ADDRESS)
            .run();

        self
    }

    pub fn deploy_chain_config(&mut self, config: SovereignConfig<StaticApi>) -> &mut Self {
        let mut additional_stake_as_tuple = MultiValueEncoded::new();
        if let Some(additional_stake) = config.opt_additional_stake_required {
            for stake in additional_stake {
                additional_stake_as_tuple.push(MultiValue2::from((stake.token_id, stake.amount)));
            }
        }

        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .typed(ChainConfigContractProxy)
            .init(
                config.min_validators as usize,
                config.max_validators as usize,
                config.min_stake,
                OWNER_ADDRESS,
                additional_stake_as_tuple,
            )
            .code(CHAIN_CONFIG_CODE_PATH)
            .new_address(CHAIN_CONFIG_ADDRESS)
            .run();

        self
    }

    pub fn assert_expected_error_message(
        &mut self,
        response: Result<(), TxResponseStatus>,
        expected_error_message: Option<&str>,
    ) {
        match response {
            Ok(_) => assert!(
                expected_error_message.is_none(),
                "Transaction was successful, but expected error"
            ),
            Err(error) => {
                assert_eq!(expected_error_message, Some(error.message.as_str()))
            }
        }
    }

    // TODO: Add a better check balance for esdt function after check storage is fixed
    pub fn check_sc_esdt_balance<ContractObj>(
        &mut self,
        tokens: Vec<MultiValue3<TestTokenIdentifier, u64, u64>>,
        contract_address: ManagedAddress<StaticApi>,
        contract: fn() -> ContractObj,
    ) where
        ContractObj: ContractBase<Api = DebugApi> + 'static,
    {
        self.world
            .tx()
            .from(OWNER_ADDRESS)
            .to(contract_address)
            .whitebox(contract, |sc: ContractObj| {
                for token in tokens {
                    let (token_id, nonce, amount) = token.into_tuple();
                    let balance = sc
                        .blockchain()
                        .get_sc_balance(&EgldOrEsdtTokenIdentifier::esdt(token_id), nonce);
                    assert_eq!(balance, amount);
                }
            });
    }

    pub fn check_multiversx_to_sovereign_token_id_mapper_is_empty(&mut self, token_name: &str) {
        self.world
            .query()
            .to(ESDT_SAFE_ADDRESS)
            .whitebox(mvx_esdt_safe::contract_obj, |sc| {
                assert!(sc
                    .multiversx_to_sovereign_token_id_mapper(
                        &TestTokenIdentifier::new(token_name).into()
                    )
                    .is_empty());
            })
    }

    pub fn check_operation_hash_status_is_empty(
        &mut self,
        operation_hash: &ManagedBuffer<StaticApi>,
    ) {
        self.world.query().to(HEADER_VERIFIER_ADDRESS).whitebox(
            header_verifier::contract_obj,
            |sc| {
                let operation_hash_whitebox =
                    ManagedBuffer::new_from_bytes(&operation_hash.to_vec());
                let hash_of_hashes =
                    ManagedBuffer::new_from_bytes(&sha256(&operation_hash_whitebox.to_vec()));

                assert!(sc
                    .operation_hash_status(&hash_of_hashes, &operation_hash_whitebox)
                    .is_empty());
            },
        )
    }

    pub fn check_operation_hash_status(
        &mut self,
        operation_hash: &ManagedBuffer<StaticApi>,
        status: OperationHashStatus,
    ) {
        self.world.query().to(HEADER_VERIFIER_ADDRESS).whitebox(
            header_verifier::contract_obj,
            |sc| {
                let operation_hash_whitebox =
                    ManagedBuffer::new_from_bytes(&operation_hash.to_vec());
                let hash_of_hashes =
                    ManagedBuffer::new_from_bytes(&sha256(&operation_hash_whitebox.to_vec()));

                assert!(
                    sc.operation_hash_status(&hash_of_hashes, &operation_hash_whitebox)
                        .get()
                        == status
                );
            },
        )
    }
}
