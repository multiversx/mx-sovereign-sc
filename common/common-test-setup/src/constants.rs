use multiversx_sc_scenario::imports::{MxscPath, TestAddress, TestSCAddress, TestTokenIdentifier};

pub const ESDT_SAFE_ADDRESS: TestSCAddress = TestSCAddress::new("esdt-safe");
pub const FEE_MARKET_ADDRESS: TestSCAddress = TestSCAddress::new("fee-market");
pub const SOV_FEE_MARKET_ADDRESS: TestSCAddress = TestSCAddress::new("sov-fee-market");
pub const HEADER_VERIFIER_ADDRESS: TestSCAddress = TestSCAddress::new("header-verifier");
pub const CHAIN_CONFIG_ADDRESS: TestSCAddress = TestSCAddress::new("chain-config");
pub const TESTING_SC_ADDRESS: TestSCAddress = TestSCAddress::new("testing-sc");
pub const CHAIN_FACTORY_SC_ADDRESS: TestSCAddress = TestSCAddress::new("chain-factory");
pub const SOVEREIGN_FORGE_SC_ADDRESS: TestSCAddress = TestSCAddress::new("sovereign-forge");

pub const OWNER_ADDRESS: TestAddress = TestAddress::new("owner");
pub const USER_ADDRESS: TestAddress = TestAddress::new("user");
pub const INSUFFICIENT_WEGLD_ADDRESS: TestAddress = TestAddress::new("insufficient_wegld");
pub const RECEIVER_ADDRESS: TestAddress = TestAddress::new("receiver");
pub const TESTING_SC: &str = "Testing SC";
pub const USER_ADDRESS_STR: &str = "User Address";
pub const MVX_ESDT_SAFE_SHARD_0: &str = "MVX ESDT Safe Shard 0";
pub const MVX_ESDT_SAFE_SHARD_1: &str = "MVX ESDT Safe Shard 1";
pub const MVX_ESDT_SAFE_SHARD_2: &str = "MVX ESDT Safe Shard 2";
pub const UNKNOWN_MVX_ESDT_SAFE: &str = "Unknown MVX ESDT Safe";
pub const FEE_MARKET_SHARD_0: &str = "Fee Market Shard 0";
pub const FEE_MARKET_SHARD_1: &str = "Fee Market Shard 1";
pub const FEE_MARKET_SHARD_2: &str = "Fee Market Shard 2";
pub const UNKNOWN_FEE_MARKET: &str = "Unknown Fee Market";

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
pub const SOV_REGISTRAR_CODE_PATH: MxscPath =
    MxscPath::new("../sov-registrar/output/sov-registrar.mxsc.json");

pub const FEE_TOKEN: TestTokenIdentifier = TestTokenIdentifier::new("INTERNS-eaad15");
pub const FIRST_TEST_TOKEN: TestTokenIdentifier = TestTokenIdentifier::new("GREEN-0e161c");
pub const SECOND_TEST_TOKEN: TestTokenIdentifier = TestTokenIdentifier::new("LTST-4f849e");
pub const NATIVE_TEST_TOKEN: TestTokenIdentifier = TestTokenIdentifier::new("NATIVE-123456");
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
pub const STATE_FILE: &str = "state.toml";
pub const NATIVE_TOKEN_TICKER: &str = "SOV";
pub const NATIVE_TOKEN_NAME: &str = "SovereignToken";

pub const ISSUE_COST: u64 = 50_000_000_000_000_000; // 0.05 EGLD
pub const GAS_LIMIT: u64 = 90_000_000; // 90 million gas limit
pub const ONE_HUNDRED_MILLION: u32 = 100_000_000;
pub const ONE_HUNDRED_THOUSAND: u32 = 100_000;
pub const OWNER_BALANCE: u128 = 100_000_000_000_000_000_000_000;
pub const DEPLOY_COST: u64 = 100_000;
pub const ESDT_SAFE_BALANCE: u128 = 100_000_000_000_000_000;
pub const ONE_THOUSAND_TOKENS: u128 = 1_000_000_000_000_000_000_000u128;
pub const ONE_HUNDRED_TOKENS: u128 = 100_000_000_000_000_000_000u128;
pub const TEN_TOKENS: u128 = 10_000_000_000_000_000_000u128;
pub const PER_TRANSFER: u64 = 100;
pub const PER_GAS: u64 = 1;

pub const EXECUTED_BRIDGE_OP_EVENT: &str = "executedBridgeOp";
pub const DEPOSIT_EVENT: &str = "deposit";
pub const SC_CALL_EVENT: &str = "scCall";
pub const REGISTER_TOKEN_EVENT: &str = "registerToken";

pub const WALLET_SHARD_0: &str = "wallets/wallet_shard_0.pem";
pub const FAILED_TO_LOAD_WALLET_SHARD_0: &str = "Failed to load wallet for shard 0";
