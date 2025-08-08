use multiversx_sc_scenario::imports::{MxscPath, TestAddress, TestSCAddress, TestTokenIdentifier};

pub const ESDT_SAFE_ADDRESS: TestSCAddress = TestSCAddress::new("esdt-safe");
pub const FEE_MARKET_ADDRESS: TestSCAddress = TestSCAddress::new("fee-market");
pub const HEADER_VERIFIER_ADDRESS: TestSCAddress = TestSCAddress::new("header-verifier");
pub const CHAIN_CONFIG_ADDRESS: TestSCAddress = TestSCAddress::new("chain-config");
pub const TESTING_SC_ADDRESS: TestSCAddress = TestSCAddress::new("testing-sc");
pub const ENSHRINE_SC_ADDRESS: TestSCAddress = TestSCAddress::new("enshrine");
pub const CHAIN_FACTORY_SC_ADDRESS: TestSCAddress = TestSCAddress::new("chain-factory");
pub const SOVEREIGN_FORGE_SC_ADDRESS: TestSCAddress = TestSCAddress::new("sovereign-forge");
pub const TOKEN_HANDLER_SC_ADDRESS: TestSCAddress = TestSCAddress::new("token-handler");

pub const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");
pub const USER_ADDRESS: TestAddress = TestAddress::new("user");
pub const INSUFFICIENT_WEGLD_ADDRESS: TestAddress = TestAddress::new("insufficient_wegld");
pub const RECEIVER_ADDRESS: TestAddress = TestAddress::new("receiver");

pub const FEE_MARKET_CODE_PATH: MxscPath =
    MxscPath::new("../fee-market/output/fee-market.mxsc.json");
pub const HEADER_VERIFIER_CODE_PATH: MxscPath =
    MxscPath::new("../header-verifier/output/header-verifier.mxsc.json");
pub const CHAIN_CONFIG_CODE_PATH: MxscPath =
    MxscPath::new("../chain-config/output/chain-config.mxsc.json");
pub const TESTING_SC_CODE_PATH: MxscPath =
    MxscPath::new("../testing-sc/output/testing-sc.mxsc.json");
pub const MVX_ESDT_SAFE_CODE_PATH: MxscPath =
    MxscPath::new("../mvx-esdt-safe/output/mvx-esdt-safe.mxsc.json");
pub const SOV_ESDT_SAFE_CODE_PATH: MxscPath =
    MxscPath::new("../sov-esdt-safe/output/to-sovereign.mxsc.json");
pub const CHAIN_FACTORY_CODE_PATH: MxscPath =
    MxscPath::new("../chain-factory/output/chain-factory.mxsc.json");
pub const SOVEREIGN_FORGE_CODE_PATH: MxscPath =
    MxscPath::new("../sovereign-forge/output/sovereign-forge.mxsc.json");
pub const ENSHRINE_ESDT_SAFE_CODE_PATH: MxscPath =
    MxscPath::new("../enshrine-esdt-safe/output/enshrine-esdt-safe.mxsc.json");
pub const TOKEN_HANDLER_CODE_PATH: MxscPath =
    MxscPath::new("../token-handler/output/token-handler.mxsc.json");

pub const FEE_TOKEN: TestTokenIdentifier = TestTokenIdentifier::new("INTERNS-eaad15");
pub const FIRST_TEST_TOKEN: TestTokenIdentifier = TestTokenIdentifier::new("GREEN-0e161c");
pub const SECOND_TEST_TOKEN: TestTokenIdentifier = TestTokenIdentifier::new("LTST-4f849e");
pub const SOV_TOKEN: TestTokenIdentifier = TestTokenIdentifier::new("sov-GREEN-0e161c");
pub const TOKEN_TICKER: &str = "GREEN";
pub const TOKEN_DISPLAY_NAME: &str = "Sovereign";
pub const REGISTER_TOKEN_PREFIX: &str = "sov-";
pub const REGISTER_DEFAULT_TOKEN: &str = "SOV-123456";
pub const NFT_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("NFT-123456");
pub const CROWD_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("CROWD-123456");
pub const FUNGIBLE_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("FUNG-123456");
pub const PREFIX_NFT_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("sov-NFT-123456");
pub const WEGLD_IDENTIFIER: TestTokenIdentifier = TestTokenIdentifier::new("WEGLD-123456");
pub const WRONG_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("WRONG-TOKEN");

pub const SOVEREIGN_RECEIVER_ADDRESS: TestAddress =
    TestAddress::new("erd18tudnj2z8vjh0339yu3vrkgzz2jpz8mjq0uhgnmklnap6z33qqeszq2yn4");

pub const SOV_TO_MVX_TOKEN_STORAGE_KEY: &str = "sovToMvxTokenId";
pub const NATIVE_TOKEN_STORAGE_KEY: &str = "nativeToken";
pub const MVX_TO_SOV_TOKEN_STORAGE_KEY: &str = "mvxToSovTokenId";
pub const OPERATION_HASH_STATUS_STORAGE_KEY: &str = "operationHashStatus";
pub const SOVEREIGN_TOKEN_PREFIX: &str = "sov";
pub const CHAIN_ID: &str = "svch";
pub const INTERACTOR_WORKING_DIR: &str = "interactor";
pub const WRONG_ENDPOINT_NAME: &str = "WRONG-ENDPOINT-NAME";
pub const ESDT_SAFE_CONFIG_STORAGE_KEY: &str = "crossChainConfig";
pub const TOKEN_FEE_STORAGE_KEY: &str = "tokenFee";
pub const NUMBER_OF_SHARDS: u32 = 3;
pub const PREFERRED_CHAIN_IDS: [&str; 3] = ["shd0", "shd1", "shd2"];
pub const SHARD_0: u32 = 0;
pub const SHARD_1: u32 = 1;
pub const SHARD_2: u32 = 2;
pub const DEPOSIT_LOG: &str = "deposit";
pub const UNPAUSE_CONTRACT_LOG: &str = "unpauseContract";
pub const TESTING_SC_ENDPOINT: &str = "hello";
pub const EXECUTED_BRIDGE_LOG: &str = "executedBridgeOp";
pub const SC_CALL_LOG: &str = "scCall";

pub const ISSUE_COST: u64 = 50_000_000_000_000_000; // 0.05 EGLD
pub const GAS_LIMIT: u64 = 90_000_000; // 90 million gas limit
pub const ONE_HUNDRED_MILLION: u32 = 100_000_000;
pub const ONE_HUNDRED_THOUSAND: u32 = 100_000;
pub const OWNER_BALANCE: u128 = 100_000_000_000_000_000_000_000;
pub const DEPLOY_COST: u64 = 100_000;
pub const ENSHRINE_BALANCE: u128 = 100_000_000_000_000_000;
pub const ONE_THOUSAND_TOKENS: u128 = 1_000_000_000_000_000_000_000u128;
pub const ONE_HUNDRED_TOKENS: u128 = 100_000_000_000_000_000_000u128;
pub const TEN_TOKENS: u128 = 10_000_000_000_000_000_000u128;
pub const PER_TRANSFER: u64 = 100;
pub const PER_GAS: u64 = 1;
