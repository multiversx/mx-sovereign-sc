use multiversx_sc_scenario::imports::{MxscPath, TestAddress, TestSCAddress};

pub const ESDT_SAFE_ADDRESS: TestSCAddress = TestSCAddress::new("esdt-safe");
pub const FEE_MARKET_ADDRESS: TestSCAddress = TestSCAddress::new("fee-market");
pub const HEADER_VERIFIER_ADDRESS: TestSCAddress = TestSCAddress::new("header-verifier");

pub const CHAIN_CONFIG_ADDRESS: TestSCAddress = TestSCAddress::new("chain-config");

pub const TESTING_SC_ADDRESS: TestSCAddress = TestSCAddress::new("testing-sc");
pub const ENSHRINE_ADDRESS: TestAddress = TestAddress::new("enshrine");

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
pub const MVX_ESDT_SAFE_CODE_PATH: MxscPath =
    MxscPath::new("../mvx-esdt-safe/output/mvx-esdt-safe.mxsc.json");
pub const SOV_ESDT_SAFE_CODE_PATH: MxscPath =
    MxscPath::new("../sov-esdt-safe/output/to-sovereign.mxsc.json");

pub const FEE_TOKEN: &str = "INTERNS-eaad15";
pub const FIRST_TEST_TOKEN: &str = "GREEN-0e161c";
pub const SECOND_TEST_TOKEN: &str = "LTST-4f849e";
pub const SOV_TOKEN: &str = "sov-GREEN-0e161c";
pub const TOKEN_TICKER: &str = "GREEN";

pub const SOV_TO_MVX_TOKEN_STORAGE_KEY: &str = "sovToMxTokenId";
pub const MVX_TO_SOV_TOKEN_STORAGE_KEY: &str = "mxToSovTokenId";
pub const OPERATION_HASH_STATUS_STORAGE_KEY: &str = "operationHashStatus";

pub const ISSUE_COST: u64 = 50_000_000_000_000_000; // 0.05 EGLD
pub const ONE_HUNDRED_MILLION: u32 = 100_000_000;
pub const ONE_HUNDRED_THOUSAND: u32 = 100_000;
pub const OWNER_BALANCE: u128 = 100_000_000_000_000_000_000_000;
